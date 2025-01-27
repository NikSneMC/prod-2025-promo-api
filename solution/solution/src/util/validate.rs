use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

use crate::models::PromoTarget;

pub fn validate_country(country: &str) -> Result<(), ValidationError> {
    if rust_iso3166::from_alpha2(&country.to_uppercase()).is_none() {
        return Err(ValidationError::new("invalid_country_code"));
    }

    Ok(())
}

pub fn validate_countries(countries: &Vec<String>) -> Result<(), ValidationError> {
    for country in countries {
        validate_country(&country)?;
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut secure = true;

    secure &= password.chars().any(|x| matches!(x, 'a'..='z'));
    secure &= password.chars().any(|x| matches!(x, 'A'..='Z'));
    secure &= password.chars().any(|x| matches!(x, '0'..='9'));
    secure &= "@$!%*?&".chars().any(|x| password.contains(x));

    if !secure {
        return Err(ValidationError::new("Specified password is too weak!"));
    }

    Ok(())
}

pub fn validate_target(target: &PromoTarget) -> Result<(), ValidationError> {
    if target.age_from.is_some() && target.age_until.is_some() && target.age_from > target.age_until
    {
        return Err(ValidationError::new(
            "`age_from` must be less than `age_until`",
        ));
    }

    if let Some(categories) = &target.categories {
        for category in categories {
            match category.len() {
                2..=20 => (),
                _ => {
                    return Err(ValidationError::new(
                        "`categories` item length must be between 2 and 20",
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn validation_errors_to_string(errors: ValidationErrors, adder: Option<String>) -> String {
    let mut output = String::new();

    let map = errors.into_errors();

    let key_option = map.keys().next().copied();

    if let Some(field) = key_option {
        if let Some(error) = map.get(field) {
            return match error {
                ValidationErrorsKind::Struct(errors) => {
                    validation_errors_to_string(*errors.clone(), Some(format!("of item {field}")))
                }
                ValidationErrorsKind::List(list) => {
                    if let Some((index, errors)) = list.iter().next() {
                        output.push_str(&validation_errors_to_string(
                            *errors.clone(),
                            Some(format!("of list {field} with index {index}")),
                        ));
                    }

                    output
                }
                ValidationErrorsKind::Field(errors) => {
                    if let Some(error) = errors.first() {
                        if let Some(adder) = adder {
                            output.push_str(&format!(
                                "Field {} {} failed validation with error: {}",
                                field, adder, error.code
                            ));
                        } else {
                            output.push_str(&format!(
                                "Field {} failed validation with error: {}",
                                field, error.code
                            ));
                        }
                    }

                    output
                }
            };
        }
    }

    String::new()
}
