#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

/// Represents information about a patient.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Patient {
    id: u64,
    name: String,
    history: String,
    password: String,
    doctors_ids: Vec<u64>,
    hospitals_ids: Vec<u64>,
}

// Implement the 'Storable' traits
impl Storable for Patient {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

/// Represents information about a hospital.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Hospital {
    id: u64,
    name: String,
    address: String,
    password: String,
    patients_ids: Vec<u64>,
    doctors_ids: Vec<u64>,
}

impl Storable for Hospital {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

/// Represents information about a doctor.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Doctor {
    id: u64,
    name: String,
    password: String,
    hospital_id: u64,
    patient_ids: Vec<u64>,
}

impl Storable for Doctor {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the 'BoundedStorable' traits
impl BoundedStorable for Patient {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Hospital {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Doctor {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local static variables for memory management and storage
thread_local! {
    /// Memory manager for handling memory operations.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Counter for generating unique IDs.
    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    /// Storage for patients.
    static PATIENT_STORAGE: RefCell<StableBTreeMap<u64, Patient, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    /// Storage for hospitals.
    static HOSPITAL_STORAGE: RefCell<StableBTreeMap<u64, Hospital, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    /// Storage for doctors.
    static DOCTOR_STORAGE: RefCell<StableBTreeMap<u64, Doctor, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

/// Struct for payload data used in update functions.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct HospitalPayload {
    #[validate(length(min = 3))]
    name: String,
    #[validate(length(min = 3))]
    address: String,
    password: String,
    city: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct PatientPayload {
    #[validate(length(min = 3))]
    name: String,
    #[validate(length(min = 6))]
    history: String,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditPatientPayload {
    name: String,
    password: String,
    patient_id: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct AddPatientToDoctor {
    doctor_id: u64,
    patient_id: u64,
    doctor_password: String,
    patient_password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct PatientHistoryUpdate {
    doctor_id: u64,
    patient_id: u64,
    doctor_password: String,
    new_history: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditDoctor {
    name: String,
    doctor_id: u64,
    hospital_id: u64,
    doctor_password: String,
    hospital_password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct DoctorPayload {
    #[validate(length(min = 3))]
    name: String,
    hospital_id: u64,
    #[validate(length(min = 4))]
    password: String,
    hospital_password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditHospitalPayload {
    hospital_id: u64,
    name: String,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct AccessPayload {
    doctor_id: u64,
    patient_id: u64,
    doctor_password: String,
}

// Query function to get all hospitals
#[ic_cdk::query]
fn get_all_hospitals() -> Result<Vec<Hospital>, Error> {
    // Retrieve all Hospitals from the storage
    let hospital_map: Vec<(u64, Hospital)> = HOSPITAL_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Hospitals from the tuple and create a vector
    let hospitals: Vec<Hospital> = hospital_map
        .into_iter()
        .map(|(_, hospital)| hospital)
        .collect();
    Ok(hospitals)
}

// Helper function to generate a unique ID for entities
fn generate_unique_id() -> u64 {
    ID_COUNTER.with(|c| {
        let id = c.borrow_mut().get();
        id
    })
}

// Helper function to add a patient to a hospital
fn add_patient_to_hospital(hospital_id: u64, patient_id: u64) {
    HOSPITAL_STORAGE.with(|h| {
        let mut hospitals = h.borrow_mut();
        if let Some(mut hospital) = hospitals.get(&hospital_id) {
            hospital.patients_ids.push(patient_id);
            hospitals.insert(hospital_id, hospital);
        }
    });
}

// Helper function to add a doctor to a hospital
fn add_doctor_to_hospital(hospital_id: u64, doctor_id: u64) {
    HOSPITAL_STORAGE.with(|h| {
        let mut hospitals = h.borrow_mut();
        if let Some(mut hospital) = hospitals.get(&hospital_id) {
            hospital.doctors_ids.push(doctor_id);
            hospitals.insert(hospital_id, hospital);
        }
    });
}

// Helper function to add a patient to a doctor
fn add_patient_to_doctor(doctor_id: u64, patient_id: u64) {
    DOCTOR_STORAGE.with(|d| {
        let mut doctors = d.borrow_mut();
        if let Some(mut doctor) = doctors.get(&doctor_id) {
            doctor.patient_ids.push(patient_id);
            doctors.insert(doctor_id, doctor);
        }
    });
}

// Helper function to add a doctor to a patient
fn add_doctor_to_patient(doctor_id: u64, patient_id: u64) {
    PATIENT_STORAGE.with(|p| {
        let mut patients = p.borrow_mut();
        if let Some(mut patient) = patients.get(&patient_id) {
            patient.doctors_ids.push(doctor_id);
            patients.insert(patient_id, patient);
        }
    });
}

// Helper function to check if a password matches the stored password
fn validate_password(stored_password: &str, input_password: &str) -> bool {
    stored_password == input_password
}

// Helper function to validate a hospital password
fn validate_hospital_password(hospital_id: u64, input_password: &str) -> Result<(), Error> {
    HOSPITAL_STORAGE.with(|h| {
        if let Some(hospital) = h.borrow().get(&hospital_id) {
            if validate_password(&hospital.password, input_password) {
                Ok(())
            } else {
                Err(Error::InvalidPassword)
            }
        } else {
            Err(Error::HospitalNotFound)
        }
    })
}

// Helper function to validate a doctor password
fn validate_doctor_password(doctor_id: u64, input_password: &str) -> Result<(), Error> {
    DOCTOR_STORAGE.with(|d| {
        if let Some(doctor) = d.borrow().get(&doctor_id) {
            if validate_password(&doctor.password, input_password) {
                Ok(())
            } else {
                Err(Error::InvalidPassword)
            }
        } else {
            Err(Error::DoctorNotFound)
        }
    })
}

// Helper function to validate a patient password
fn validate_patient_password(patient_id: u64, input_password: &str) -> Result<(), Error> {
    PATIENT_STORAGE.with(|p| {
        if let Some(patient) = p.borrow().get(&patient_id) {
            if validate_password(&patient.password, input_password) {
                Ok(())
            } else {
                Err(Error::InvalidPassword)
            }
        } else {
            Err(Error::PatientNotFound)
        }
    })
}

// Helper function to validate access to a patient by a doctor
fn validate_patient_access(
    doctor_id: u64,
    patient_id: u64,
    doctor_password: &str,
) -> Result<(), Error> {
    validate_doctor_password(doctor_id, doctor_password)?;
    DOCTOR_STORAGE.with(|d| {
        if let Some(doctor) = d.borrow().get(&doctor_id) {
            if doctor.patient_ids.contains(&patient_id) {
                Ok(())
            } else {
                Err(Error::PatientAccessDenied)
            }
        } else {
            Err(Error::DoctorNotFound)
        }
    })
}

// Helper function to validate access to a doctor by a hospital
fn validate_doctor_access(
    hospital_id: u64,
    doctor_id: u64,
    hospital_password: &str,
) -> Result<(), Error> {
    validate_hospital_password(hospital_id, hospital_password)?;
    HOSPITAL_STORAGE.with(|h| {
        if let Some(hospital) = h.borrow().get(&hospital_id) {
            if hospital.doctors_ids.contains(&doctor_id) {
                Ok(())
            } else {
                Err(Error::DoctorAccessDenied)
            }
        } else {
            Err(Error::HospitalNotFound)
        }
    })
}

// Helper function to validate access to a patient by a hospital
fn validate_patient_hospital_access(
    hospital_id: u64,
    patient_id: u64,
    hospital_password: &str,
) -> Result<(), Error> {
    validate_hospital_password(hospital_id, hospital_password)?;
    HOSPITAL_STORAGE.with(|h| {
        if let Some(hospital) = h.borrow().get(&hospital_id) {
            if hospital.patients_ids.contains(&patient_id) {
                Ok(())
            } else {
                Err(Error::PatientHospitalAccessDenied)
            }
        } else {
            Err(Error::HospitalNotFound)
        }
    })
}

/// Error types that can be returned by the canister functions.
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
enum Error {
    #[serde(rename = "PatientNotFound")]
    PatientNotFound,
    #[serde(rename = "DoctorNotFound")]
    DoctorNotFound,
    #[serde(rename = "HospitalNotFound")]
    HospitalNotFound,
    #[serde(rename = "InvalidPassword")]
    InvalidPassword,
    #[serde(rename = "PatientAccessDenied")]
    PatientAccessDenied,
    #[serde(rename = "DoctorAccessDenied")]
    DoctorAccessDenied,
    #[serde(rename = "PatientHospitalAccessDenied")]
    PatientHospitalAccessDenied,
    #[serde(rename = "ValidationFailed")]
    ValidationFailed(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

/// Initializes the canister.
#[ic_cdk::init]
fn init() {
    // Initialize the memory manager
    MEMORY_MANAGER.with(|m| *m.borrow_mut() = MemoryManager::init(DefaultMemoryImpl::default()));

    // Initialize the ID counter
    ID_COUNTER.with(|c| *c.borrow_mut() = IdCell::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        0,
    ).expect("Cannot create a counter"));

    // Initialize the storage for patients
    PATIENT_STORAGE.with(|p| *p.borrow_mut() = StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
    ));

    // Initialize the storage for hospitals
    HOSPITAL_STORAGE.with(|h| *h.borrow_mut() = StableBTreeMap::init(
        MEMORY_MANAGER.with(        |m| m.borrow().get(MemoryId::new(3))),
    ));

    // Initialize the storage for doctors
    DOCTOR_STORAGE.with(|d| *d.borrow_mut() = StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
    ));
}

// Function to create a new hospital
#[ic_cdk::update]
fn create_hospital(payload: HospitalPayload) -> Result<u64, Error> {
    let hospital_id = generate_unique_id();

    let new_hospital = Hospital {
        id: hospital_id,
        name: payload.name,
        address: payload.address,
        password: payload.password,
        patients_ids: Vec::new(),
        doctors_ids: Vec::new(),
    };

    HOSPITAL_STORAGE.with(|h| {
        let mut hospitals = h.borrow_mut();
        hospitals.insert(hospital_id, new_hospital.clone());
    });

    Ok(hospital_id)
}

// Function to create a new patient
#[ic_cdk::update]
fn create_patient(
    hospital_id: u64,
    payload: PatientPayload,
    hospital_password: String,
) -> Result<u64, Error> {
    validate_hospital_password(hospital_id, &hospital_password)?;

    let patient_id = generate_unique_id();

    let new_patient = Patient {
        id: patient_id,
        name: payload.name,
        history: payload.history,
        password: payload.password,
        doctors_ids: Vec::new(),
        hospitals_ids: vec![hospital_id],
    };

    PATIENT_STORAGE.with(|p| {
        let mut patients = p.borrow_mut();
        patients.insert(patient_id, new_patient.clone());
    });

    add_patient_to_hospital(hospital_id, patient_id);

    Ok(patient_id)
}

// Function to create a new doctor
#[ic_cdk::update]
fn create_doctor(
    hospital_id: u64,
    payload: DoctorPayload,
    hospital_password: String,
) -> Result<u64, Error> {
    validate_hospital_password(hospital_id, &hospital_password)?;

    let doctor_id = generate_unique_id();

    let new_doctor = Doctor {
        id: doctor_id,
        name: payload.name,
        password: payload.password,
        hospital_id,
        patient_ids: Vec::new(),
    };

    DOCTOR_STORAGE.with(|d| {
        let mut doctors = d.borrow_mut();
        doctors.insert(doctor_id, new_doctor.clone());
    });

    add_doctor_to_hospital(hospital_id, doctor_id);

    Ok(doctor_id)
}

// Function to get patient details
#[ic_cdk::query]
fn get_patient_details(
    patient_id: u64,
    patient_password: String,
) -> Result<Patient, Error> {
    validate_patient_password(patient_id, &patient_password)?;
    PATIENT_STORAGE.with(|p| {
        if let Some(patient) = p.borrow().get(&patient_id) {
            Ok(patient.clone())
        } else {
            Err(Error::PatientNotFound)
        }
    })
}

// Function to get doctor details
#[ic_cdk::query]
fn get_doctor_details(
    doctor_id: u64,
    hospital_id: u64,
    doctor_password: String,
    hospital_password: String,
) -> Result<Doctor, Error> {
    validate_doctor_access(hospital_id, doctor_id, &hospital_password)?;
    DOCTOR_STORAGE.with(|d| {
        if let Some(doctor) = d.borrow().get(&doctor_id) {
            Ok(doctor.clone())
        } else {
            Err(Error::DoctorNotFound)
        }
    })
}

// Function to edit patient details
#[ic_cdk::update]
fn edit_patient_details(
    payload: EditPatientPayload,
    patient_password: String,
) -> Result<(), Error> {
    validate_patient_password(payload.patient_id, &patient_password)?;

    PATIENT_STORAGE.with(|p| {
        if let Some(mut patient) = p.borrow_mut().get_mut(&payload.patient_id) {
            patient.name = payload.name;
            patient.password = payload.password;
            p.borrow_mut().insert(payload.patient_id, patient);
            Ok(())
        } else {
            Err(Error::PatientNotFound)
        }
    })
}

// Function to edit doctor details
#[ic_cdk::update]
fn edit_doctor_details(
    payload: EditDoctor,
    doctor_password: String,
    hospital_password: String,
) -> Result<(), Error> {
    validate_doctor_access(payload.hospital_id, payload.doctor_id, &hospital_password)?;

    DOCTOR_STORAGE.with(|d| {
        if let Some(mut doctor) = d.borrow_mut().get_mut(&payload.doctor_id) {
            doctor.name = payload.name;
            doctor.password = payload.doctor_password;
            d.borrow_mut().insert(payload.doctor_id, doctor);
            Ok(())
        } else {
            Err(Error::DoctorNotFound)
        }
    })
}

// Function to add a patient to a doctor
#[ic_cdk::update]
fn add_patient_to_doctor_relationship(
    payload: AddPatientToDoctor,
) -> Result<(), Error> {
    validate_doctor_password(payload.doctor_id, &payload.doctor_password)?;
    validate_patient_password(payload.patient_id, &payload.patient_password)?;

    add_patient_to_doctor(payload.doctor_id, payload.patient_id);
    add_doctor_to_patient(payload.doctor_id, payload.patient_id);

    Ok(())
}

// Function to update patient history by a doctor
#[ic_cdk::update]
fn update_patient_history_by_doctor(
    payload: PatientHistoryUpdate,
) -> Result<(), Error> {
    validate_doctor_password(payload.doctor_id, &payload.doctor_password)?;
    validate_patient_access(payload.doctor_id, payload.patient_id, &payload.doctor_password)?;

    PATIENT_STORAGE.with(|p| {
        if let Some(mut patient) = p.borrow_mut().get_mut(&payload.patient_id) {
            patient.history = payload.new_history;
            p.borrow_mut().insert(payload.patient_id, patient);
            Ok(())
        } else {
            Err(Error::PatientNotFound)
        }
    })
}

/// Function to get the timestamp of the current block.
fn current_time() -> u64 {
    time()
}

/// Error types that can be returned by the canister functions.
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
enum TimestampError {
    #[serde(rename = "InvalidTimestamp")]
    InvalidTimestamp,
}

impl std::fmt::Display for TimestampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TimestampError {}

// Function to get the current timestamp
#[ic_cdk::query]
fn get_current_timestamp() -> Result<u64, TimestampError> {
    Ok(current_time())
}

// Function to get the patients associated with a doctor
#[ic_cdk::query]
fn get_patients_for_doctor(
    doctor_id: u64,
    doctor_password: String,
) -> Result<Vec<Patient>, Error> {
    validate_doctor_password(doctor_id, &doctor_password)?;
    DOCTOR_STORAGE.with(|d| {
        if let Some(doctor) = d.borrow().get(&doctor_id) {
            let patient_ids = &doctor.patient_ids;
            PATIENT_STORAGE
                .with(|p| Ok(patient_ids.iter().flat_map(|id| p.borrow().get(id)).cloned().collect()))
        } else {
            Err(Error::DoctorNotFound)
        }
    })
}

// Function to get the doctors associated with a hospital
#[ic_cdk::query]
fn get_doctors_for_hospital(
    hospital_id: u64,
    hospital_password: String,
) -> Result<Vec<Doctor>, Error> {
    validate_hospital_password(hospital_id, &hospital_password)?;
    HOSPITAL_STORAGE.with(|h| {
        if let Some(hospital) = h.borrow().get(&hospital_id) {
            let doctor_ids = &hospital.doctors_ids;
            DOCTOR_STORAGE
                .with(|d| Ok(doctor_ids.iter().flat_map(|id| d.borrow().get(id)).cloned().collect()))
        } else {
            Err(Error::HospitalNotFound)
        }
    })
}

// Function to get the hospitals associated with a patient
#[ic_cdk::query]
fn get_hospitals_for_patient(
    patient_id: u64,
    patient_password: String,
) -> Result<Vec<Hospital>, Error> {
    validate_patient_password(patient_id, &patient_password)?;
    PATIENT_STORAGE.with(|p| {
        if let Some(patient) = p.borrow().get(&patient_id) {
            let hospital_ids = &patient.hospitals_ids;
            HOSPITAL_STORAGE
                .with(|h| Ok(hospital_ids.iter().flat_map(|id| h.borrow().get(id)).cloned().collect()))
        } else {
            Err(Error::PatientNotFound)
        }
    })
}

// Entry point for timestamp queries
#[ic_cdk::query]
fn __get_candid_interface_tmp_hack() -> String {
    __export_point!()
}

// Entry point for update calls
#[ic_cdk::update]
fn __execute_update_tmp_hack() -> String {
    __export_point!()
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
