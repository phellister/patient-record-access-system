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
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

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
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Doctor {
    id: u64,
    name: String,
    password: String,
    hospital_id: u64,
    patient_ids: Vec<u64>,
}

impl Storable for Doctor {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
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
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static PATIENT_STORAGE: RefCell<StableBTreeMap<u64, Patient, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static HOSPITAL_STORAGE: RefCell<StableBTreeMap<u64, Hospital, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static DOCTOR_STORAGE: RefCell<StableBTreeMap<u64, Doctor, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

// Struct for payload date used in update functions
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
        .map(|hospital| Hospital {
            password: "-".to_string(),
            ..hospital
        })
        .collect();

    match hospitals.len() {
        0 => Err(Error::NotFound {
            msg: format!("no Hospitals found"),
        }),
        _ => Ok(hospitals),
    }
}

// Get Hospitals by city and name content
#[ic_cdk::query]
fn get_hospital_by_name(search: String) -> Result<Vec<Hospital>, Error> {
    let query = search.to_lowercase();
    // Retrieve all Hospitals from the storage
    let hospital_map: Vec<(u64, Hospital)> = HOSPITAL_STORAGE.with(|s| s.borrow().iter().collect());
    let hospitals: Vec<Hospital> = hospital_map
        .into_iter()
        .map(|(_, hospital)| hospital)
        .collect();

    // Filter the hospitals by name
    let incomplete_patients: Vec<Hospital> = hospitals
        .into_iter()
        .filter(|hospital| (hospital.name).to_lowercase().contains(&query))
        .map(|hospital| Hospital {
            password: "-".to_string(),
            ..hospital
        })
        .collect();

    // Check if any hospitals are found
    match incomplete_patients.len() {
        0 => Err(Error::NotFound {
            msg: format!("No hospitals for name: {} could be found", query),
        }),
        _ => Ok(incomplete_patients),
    }
}

// get hospital by ID
#[ic_cdk::query]
fn get_hospital_by_id(id: u64) -> Result<Hospital, Error> {
    match HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&id)) {
        Some(hospital) => Ok(Hospital {
            password: "-".to_string(),
            ..hospital
        }),
        None => Err(Error::NotFound {
            msg: format!("hospital of id: {} not found", id),
        }),
    }
}

// Create new Hospital
#[ic_cdk::update]
fn add_hospital(payload: HospitalPayload) -> Result<Hospital, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let hospital = Hospital {
        id,
        name: payload.name.clone(),
        address: payload.address,
        password: payload.password,
        patients_ids: vec![],
        doctors_ids: vec![],
    };

    match HOSPITAL_STORAGE.with(|s| s.borrow_mut().insert(id, hospital.clone())) {
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add hospital name: {}", payload.name),
        }),
        None => Ok(hospital),
    }
}

// update function to edit a hospital where only owners of hospitals can edit title, is_community, price and description. Non owners can only edit descriptions of communtiy hospitals. authorizations is by password
#[ic_cdk::update]
fn edit_hospital(payload: EditHospitalPayload) -> Result<Hospital, Error> {
    let hospital = HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&payload.hospital_id));

    match hospital {
        Some(hospital) => {
            // check if the password provided matches hospital
            if hospital.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            let new_hospital = Hospital {
                name: payload.name,
                ..hospital.clone()
            };

            match HOSPITAL_STORAGE
                .with(|s| s.borrow_mut().insert(hospital.id, new_hospital.clone()))
            {
                Some(_) => Ok(new_hospital),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not edit hospital title: {}", hospital.name),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("hospital of id: {} not found", payload.hospital_id),
        }),
    }
}

// function to assign patient to doctor and add patient to doctor's hospital
#[ic_cdk::update]
fn assign_patient_to_doctor(payload: AddPatientToDoctor) -> Result<String, Error> {
    // get patient
    let doctor = DOCTOR_STORAGE.with(|doctors| doctors.borrow().get(&payload.doctor_id));
    match doctor {
        Some(doctor) => {
            if doctor.password != payload.doctor_password {
                return Err(Error::Unauthorized {
                    msg: format!("Doctor Access unauthorized, password does not match, try again"),
                });
            }
            let patient =
                PATIENT_STORAGE.with(|patients| patients.borrow().get(&payload.patient_id));
            match patient {
                Some(patient) => {
                    // check if the password provided matches patient
                    if patient.password != payload.patient_password {
                        return Err(Error::Unauthorized {
                            msg: format!(
                                "Patient access unauthorized, password does not match, try again"
                            ),
                        });
                    }
                    let mut new_doctor_patient_ids = doctor.patient_ids.clone();
                    new_doctor_patient_ids.push(patient.id);
                    let new_doctor = Doctor {
                        patient_ids: new_doctor_patient_ids,
                        name: doctor.name.clone(),
                        ..doctor
                    };
                    // add patient to hospital
                    match add_patient_to_hospital(doctor.hospital_id, patient.id) {
                        Ok(_) => {
                            // update doctor in storage
                            match DOCTOR_STORAGE
                                .with(|s| s.borrow_mut().insert(doctor.id, new_doctor.clone()))
                            {
                                Some(_) => {
                                    // update patient
                                    let mut new_patient_doctors_ids = patient.doctors_ids.clone();
                                    new_patient_doctors_ids.push(doctor.id);
                                    let new_patient = Patient {
                                        doctors_ids: new_patient_doctors_ids,
                                        ..patient.clone()
                                    };
                                    // update patient in storage
                                    match PATIENT_STORAGE
                                        .with(|s| s.borrow_mut().insert(patient.id, new_patient.clone()))
                                    {
                                        Some(_) => Ok(format!(
                                            "Succesfully assigned patient {} to doctor: {} and hospital: {} ",
                                            patient.name, doctor.name, doctor.hospital_id
                                        )),
                                        None => Err(Error::InvalidPayload {
                                            msg: format!("Could not update patient"),
                                        }),
                                    }
                                }
                                None => Err(Error::InvalidPayload {
                                    msg: format!("Could not update doctor"),
                                }),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("Doctor of id: {} not found", payload.doctor_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("patient of id: {} not found", payload.patient_id),
        }),
    }
}

// helper function to add patient to hospital
fn add_patient_to_hospital(hospital_id: u64, patient_id: u64) -> Result<(), Error> {
    // get hospital
    let hospital = HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&hospital_id));
    // get doctor
    match hospital {
        Some(hospital) => {
            // add patient Id to hospital patients
            let mut new_hospital_patients_ids = hospital.patients_ids.clone();
            new_hospital_patients_ids.push(patient_id);
            let new_hospital = Hospital {
                patients_ids: new_hospital_patients_ids,
                ..hospital.clone()
            };
            // update hospital in storage
            match HOSPITAL_STORAGE
                .with(|s| s.borrow_mut().insert(hospital.id, new_hospital.clone()))
            {
                Some(_) => Ok(()),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not update hospital"),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("hospital of id: {} not found", hospital_id),
        }),
    }
}

// function to add to patients medical history by patient's doctor. authorizations is by doctor password
#[ic_cdk::update]
fn update_patient_history(payload: PatientHistoryUpdate) -> Result<String, Error> {
    // get patient
    let doctor = DOCTOR_STORAGE.with(|doctors| doctors.borrow().get(&payload.doctor_id));
    match doctor {
        Some(doctor) => {
            if doctor.password != payload.doctor_password {
                return Err(Error::Unauthorized {
                    msg: format!("Doctor Access unauthorized, password does not match, try again"),
                });
            }
            let patient =
                PATIENT_STORAGE.with(|patients| patients.borrow().get(&payload.patient_id));
            match patient {
                Some(patient) => {
                    // check if the password provided matches patient
                    if patient.doctors_ids.contains(&doctor.id) == false {
                        return Err(Error::Unauthorized {
                            msg: format!(
                                "Patient access unauthorized, doctor is not assigned to patient, try again"
                            ),
                        });
                    }
                    let new_patient = Patient {
                        // add new history to patient.history with current time and doctor id
                        history: format!(
                            "{} \n Doctor {} : {} at {} \n {}",
                            patient.history,
                            doctor.id,
                            { doctor.name },
                            time(),
                            payload.new_history
                        ),
                        ..patient.clone()
                    };
                    // update patient in storage
                    match PATIENT_STORAGE
                        .with(|s| s.borrow_mut().insert(patient.id, new_patient.clone()))
                    {
                        Some(_) => Ok(format!(
                            "Succesfully updated patient {} history",
                            patient.name
                        )),
                        None => Err(Error::InvalidPayload {
                            msg: format!("Could not update patient"),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("Patient of id: {} not found", payload.patient_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("Doctor of id: {} not found", payload.doctor_id),
        }),
    }
}

// Define query function to get a patient by ID
#[ic_cdk::query]
fn get_patient(id: u64) -> Result<Patient, Error> {
    match PATIENT_STORAGE.with(|patients| patients.borrow().get(&id)) {
        Some(patient) => Ok(Patient {
            password: "-".to_string(),
            history: "-".to_string(),
            ..patient
        }),
        None => Err(Error::NotFound {
            msg: format!("patient id:{} does not exist", id),
        }),
    }
}

// query function for doctor to get patient info by patient id and doctor password
#[ic_cdk::query]
fn get_patient_info(payload: AccessPayload) -> Result<Patient, Error> {
    // get patient
    let doctor = DOCTOR_STORAGE.with(|doctors| doctors.borrow().get(&payload.doctor_id));
    match doctor {
        Some(doctor) => {
            if doctor.password != payload.doctor_password {
                return Err(Error::Unauthorized {
                    msg: format!("Doctor Access unauthorized, password does not match, try again"),
                });
            }
            let patient =
                PATIENT_STORAGE.with(|patients| patients.borrow().get(&payload.patient_id));
            match patient {
                Some(patient) => {
                    // check if the password provided matches patient
                    if patient.doctors_ids.contains(&doctor.id) == false {
                        return Err(Error::Unauthorized {
                            msg: format!(
                                "Patient access unauthorized, doctor is not assigned to patient, get patient permission"
                            ),
                        });
                    }
                    Ok(Patient {
                        password: "-".to_string(),
                        ..patient.clone()
                    })
                }
                None => Err(Error::NotFound {
                    msg: format!("Doctor of id: {} not found", payload.doctor_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("patient of id: {} not found", payload.patient_id),
        }),
    }
}

// Update function to add a patient
#[ic_cdk::update]
fn add_patient(payload: PatientPayload) -> Result<Patient, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let patient = Patient {
        id,
        name: payload.name.clone(),
        history: payload.history,
        password: payload.password,
        doctors_ids: vec![],
        hospitals_ids: vec![],
    };

    match PATIENT_STORAGE.with(|s| s.borrow_mut().insert(id, patient.clone())) {
        None => Ok(patient),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add patient name: {}", payload.name),
        }),
    }
}

// add patient to hospital

// update function to edit a patient where authorizations is by password
#[ic_cdk::update]
fn edit_patient(payload: EditPatientPayload) -> Result<Patient, Error> {
    let patient = PATIENT_STORAGE.with(|patients| patients.borrow().get(&payload.patient_id));

    match patient {
        Some(patient) => {
            // check if the password provided matches patient
            if patient.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            let new_patient = Patient {
                name: payload.name,
                ..patient.clone()
            };

            match PATIENT_STORAGE.with(|s| s.borrow_mut().insert(patient.id, new_patient.clone())) {
                Some(_) => Ok(new_patient),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not edit patient name: {}", patient.name),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("patient of id: {} not found", payload.patient_id),
        }),
    }
}

// get doctor by ID
#[ic_cdk::query]
fn get_doctor_by_id(id: u64) -> Result<Doctor, Error> {
    match DOCTOR_STORAGE.with(|doctors| doctors.borrow().get(&id)) {
        Some(doctor) => Ok(Doctor {
            password: "-".to_string(),
            ..doctor
        }),
        None => Err(Error::NotFound {
            msg: format!("doctor id:{} does not exist", id),
        }),
    }
}

// add doctor to hospital
#[ic_cdk::update]
fn add_doctor(payload: DoctorPayload) -> Result<Doctor, Error> {
    let hospital = HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&payload.hospital_id));
    match hospital {
        Some(hospital) => {
            // check if the password provided matches hospital
            if hospital.password != payload.hospital_password {
                return Err(Error::Unauthorized {
                    msg: format!(
                        "Hospital access unauthorized, password does not match, try again"
                    ),
                });
            }
            let validate_payload = payload.validate();
            if validate_payload.is_err() {
                return Err(Error::InvalidPayload {
                    msg: validate_payload.unwrap_err().to_string(),
                });
            }

            match add_doctor_to_storage(payload.clone()) {
                Ok(doctor) => match add_doctor_to_hospital(doctor, hospital.clone()) {
                    Ok(response) => Ok(response),
                    Err(e) => Err(e),
                },
                Err(e) => return Err(e),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("Hospital of id: {} not found", payload.hospital_id),
        }),
    }
}

// helper function to add doctor to storage
fn add_doctor_to_storage(payload: DoctorPayload) -> Result<Doctor, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let doctor = Doctor {
        id,
        name: payload.name.clone(),
        hospital_id: payload.hospital_id,
        password: payload.password,
        patient_ids: vec![],
    };
    match DOCTOR_STORAGE.with(|s| s.borrow_mut().insert(id, doctor.clone())) {
        None => Ok(doctor),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add doctor name: {}", payload.name),
        }),
    }
}

fn add_doctor_to_hospital(doctor: Doctor, hospital: Hospital) -> Result<Doctor, Error> {
    let mut new_hospital_doctors_ids = hospital.doctors_ids.clone();
    new_hospital_doctors_ids.push(doctor.id);
    let new_hospital = Hospital {
        doctors_ids: new_hospital_doctors_ids,
        name: hospital.name.clone(),
        ..hospital.clone()
    };
    // update hospital in storage
    match HOSPITAL_STORAGE.with(|s| s.borrow_mut().insert(hospital.id, new_hospital.clone())) {
        Some(_) => {
            // update doctor
            let new_doctor = Doctor {
                hospital_id: hospital.id,
                name: doctor.name.clone(),
                ..doctor.clone()
            };
            // update doctor in storage
            match DOCTOR_STORAGE.with(|s| s.borrow_mut().insert(doctor.id, new_doctor.clone())) {
                Some(_) => Ok(doctor),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not update doctor"),
                }),
            }
        }
        None => Err(Error::InvalidPayload {
            msg: format!("Could not update hospital"),
        }),
    }
}

// add doctor to hospital
#[ic_cdk::update]
fn edit_doctor(payload: EditDoctor) -> Result<String, Error> {
    // get doctor
    let doctor = DOCTOR_STORAGE.with(|doctors| doctors.borrow().get(&payload.doctor_id));
    match doctor {
        Some(doctor) => {
            if doctor.password != payload.doctor_password {
                return Err(Error::Unauthorized {
                    msg: format!("Doctor Access unauthorized, password does not match, try again"),
                });
            }
            let hospital =
                HOSPITAL_STORAGE.with(|hospitals| hospitals.borrow().get(&payload.hospital_id));
            match hospital {
                Some(hospital) => {
                    // check if the password provided matches hospital
                    if hospital.password != payload.hospital_password {
                        return Err(Error::Unauthorized {
                            msg: format!(
                                "Hospital access unauthorized, password does not match, try again"
                            ),
                        });
                    }
                    let mut new_hospital_doctors_ids = hospital.doctors_ids.clone();
                    new_hospital_doctors_ids.push(doctor.id);
                    let new_hospital = Hospital {
                        doctors_ids: new_hospital_doctors_ids,
                        name: hospital.name.clone(),
                        ..hospital.clone()
                    };
                    // update hospital in storage
                    match HOSPITAL_STORAGE
                        .with(|s| s.borrow_mut().insert(hospital.id, new_hospital.clone()))
                    {
                        Some(_) => {
                            // update doctor
                            let new_doctor = Doctor {
                                hospital_id: hospital.id,
                                name: payload.name.clone(),
                                ..doctor.clone()
                            };
                            // update doctor in storage
                            match DOCTOR_STORAGE
                                .with(|s| s.borrow_mut().insert(doctor.id, new_doctor.clone()))
                            {
                                Some(_) => Ok(format!(
                                    "Succesfully assigned doctor {} to hospital: {} ",
                                    payload.name, hospital.name
                                )),
                                None => Err(Error::InvalidPayload {
                                    msg: format!("Could not update doctor"),
                                }),
                            }
                        }
                        None => Err(Error::InvalidPayload {
                            msg: format!("Could not update hospital"),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("Hospital of id: {} not found", payload.hospital_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("doctor of id: {} not found", payload.doctor_id),
        }),
    }
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyInit { msg: String },
    InvalidPayload { msg: String },
    Unauthorized { msg: String },
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
