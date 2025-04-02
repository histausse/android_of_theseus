use androscalpel::{IdMethod, IdType};
use anyhow::{bail, Result};
use std::sync::LazyLock;

pub(crate) static MTH_INVOKE: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali(
    "Ljava/lang/reflect/Method;->invoke(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
)
.unwrap()
});
pub(crate) static MTH_GET_NAME: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getName()Ljava/lang/String;").unwrap()
});
pub(crate) static MTH_GET_PARAMS_TY: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getParameterTypes()[Ljava/lang/Class;")
        .unwrap()
});
pub(crate) static MTH_GET_RET_TY: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getReturnType()Ljava/lang/Class;").unwrap()
});
pub(crate) static MTH_GET_DEC_CLS: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getDeclaringClass()Ljava/lang/Class;")
        .unwrap()
});
pub(crate) static STR_EQ: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/String;->equals(Ljava/lang/Object;)Z").unwrap()
});
pub(crate) static CLASS_NEW_INST: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Class;->newInstance()Ljava/lang/Object;").unwrap()
});
pub(crate) static CNSTR_NEW_INST: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali(
        "Ljava/lang/reflect/Constructor;->newInstance([Ljava/lang/Object;)Ljava/lang/Object;",
    )
    .unwrap()
});
pub(crate) static CNSTR_GET_PARAMS_TY: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Constructor;->getParameterTypes()[Ljava/lang/Class;")
        .unwrap()
});
pub(crate) static CNSTR_GET_DEC_CLS: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Constructor;->getDeclaringClass()Ljava/lang/Class;")
        .unwrap()
});
pub(crate) static CLT_GET_DESCR_STRING: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Class;->descriptorString()Ljava/lang/String;").unwrap()
});
pub(crate) static OBJ_TO_SCAL_BOOL: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Boolean;->booleanValue()Z").unwrap());
pub(crate) static OBJ_TO_SCAL_BYTE: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Byte;->byteValue()B").unwrap());
pub(crate) static OBJ_TO_SCAL_SHORT: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Short;->shortValue()S").unwrap());
pub(crate) static OBJ_TO_SCAL_CHAR: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Character;->charValue()C").unwrap());
pub(crate) static OBJ_TO_SCAL_INT: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Integer;->intValue()I").unwrap());
pub(crate) static OBJ_TO_SCAL_LONG: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Long;->longValue()J").unwrap());
pub(crate) static OBJ_TO_SCAL_FLOAT: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Float;->floatValue()F").unwrap());
pub(crate) static OBJ_TO_SCAL_DOUBLE: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Double;->doubleValue()D").unwrap());
pub(crate) static OBJ_OF_SCAL_BOOL: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Boolean;").unwrap());
pub(crate) static OBJ_OF_SCAL_BYTE: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Byte;").unwrap());
pub(crate) static OBJ_OF_SCAL_SHORT: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Short;").unwrap());
pub(crate) static OBJ_OF_SCAL_CHAR: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Character;").unwrap());
pub(crate) static OBJ_OF_SCAL_INT: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Integer;").unwrap());
pub(crate) static OBJ_OF_SCAL_LONG: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Long;").unwrap());
pub(crate) static OBJ_OF_SCAL_FLOAT: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Float;").unwrap());
pub(crate) static OBJ_OF_SCAL_DOUBLE: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Double;").unwrap());
pub(crate) static SCAL_TO_OBJ_BOOL: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Boolean;->valueOf(Z)Ljava/lang/Boolean;").unwrap()
});
pub(crate) static SCAL_TO_OBJ_BYTE: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Byte;->valueOf(B)Ljava/lang/Byte;").unwrap());
pub(crate) static SCAL_TO_OBJ_SHORT: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Short;->valueOf(S)Ljava/lang/Short;").unwrap()
});
pub(crate) static SCAL_TO_OBJ_CHAR: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Character;->valueOf(C)Ljava/lang/Character;").unwrap()
});
pub(crate) static SCAL_TO_OBJ_INT: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;").unwrap()
});
pub(crate) static SCAL_TO_OBJ_LONG: LazyLock<IdMethod> =
    LazyLock::new(|| IdMethod::from_smali("Ljava/lang/Long;->valueOf(J)Ljava/lang/Long;").unwrap());
pub(crate) static SCAL_TO_OBJ_FLOAT: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Float;->valueOf(F)Ljava/lang/Float;").unwrap()
});
pub(crate) static SCAL_TO_OBJ_DOUBLE: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Double;->valueOf(D)Ljava/lang/Double;").unwrap()
});

pub(crate) static OBJECT_TY: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ljava/lang/Object;").unwrap());
pub(crate) static DELEGATE_LAST_CLASS_LOADER: LazyLock<IdType> =
    LazyLock::new(|| IdType::from_smali("Ldalvik/system/DelegateLastClassLoader;").unwrap());

/// Get the method that convert a object to its scalar conterpart (eg `java.lang.Integer` to `int` with
/// `Ljava/lang/Integer;->intValue()I`)
///
/// `scalar_ty` is the type of the scalar (eg `I`)
pub fn get_obj_to_scalar_method(scalar_ty: &IdType) -> Result<IdMethod> {
    if scalar_ty == &IdType::boolean() {
        Ok(OBJ_TO_SCAL_BOOL.clone())
    } else if scalar_ty == &IdType::byte() {
        Ok(OBJ_TO_SCAL_BYTE.clone())
    } else if scalar_ty == &IdType::short() {
        Ok(OBJ_TO_SCAL_SHORT.clone())
    } else if scalar_ty == &IdType::char() {
        Ok(OBJ_TO_SCAL_CHAR.clone())
    } else if scalar_ty == &IdType::int() {
        Ok(OBJ_TO_SCAL_INT.clone())
    } else if scalar_ty == &IdType::long() {
        Ok(OBJ_TO_SCAL_LONG.clone())
    } else if scalar_ty == &IdType::float() {
        Ok(OBJ_TO_SCAL_FLOAT.clone())
    } else if scalar_ty == &IdType::double() {
        Ok(OBJ_TO_SCAL_DOUBLE.clone())
    } else {
        bail!("{} is not a scalar", scalar_ty.__str__())
    }
}

/// Get the object associated to a scalar (eg `java.lang.Integer` for `int`)
///
/// `scalar_ty` is the type of the scalar (eg `I`)
pub fn get_obj_of_scalar(scalar_ty: &IdType) -> Result<IdType> {
    if scalar_ty == &IdType::boolean() {
        Ok(OBJ_OF_SCAL_BOOL.clone())
    } else if scalar_ty == &IdType::byte() {
        Ok(OBJ_OF_SCAL_BYTE.clone())
    } else if scalar_ty == &IdType::short() {
        Ok(OBJ_OF_SCAL_SHORT.clone())
    } else if scalar_ty == &IdType::char() {
        Ok(OBJ_OF_SCAL_CHAR.clone())
    } else if scalar_ty == &IdType::int() {
        Ok(OBJ_OF_SCAL_INT.clone())
    } else if scalar_ty == &IdType::long() {
        Ok(OBJ_OF_SCAL_LONG.clone())
    } else if scalar_ty == &IdType::float() {
        Ok(OBJ_OF_SCAL_FLOAT.clone())
    } else if scalar_ty == &IdType::double() {
        Ok(OBJ_OF_SCAL_DOUBLE.clone())
    } else {
        bail!("{} is not a scalar", scalar_ty.__str__())
    }
}

/// Get the method that convert a scalar to its object conterpart (eg `int` to `java.lang.Integer` with
/// `Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;`)
///
/// `scalar_ty` is the type of the scalar (eg `I`)
pub fn get_scalar_to_obj_method(scalar_ty: &IdType) -> Result<IdMethod> {
    if scalar_ty == &IdType::boolean() {
        Ok(SCAL_TO_OBJ_BOOL.clone())
    } else if scalar_ty == &IdType::byte() {
        Ok(SCAL_TO_OBJ_BYTE.clone())
    } else if scalar_ty == &IdType::short() {
        Ok(SCAL_TO_OBJ_SHORT.clone())
    } else if scalar_ty == &IdType::char() {
        Ok(SCAL_TO_OBJ_CHAR.clone())
    } else if scalar_ty == &IdType::int() {
        Ok(SCAL_TO_OBJ_INT.clone())
    } else if scalar_ty == &IdType::long() {
        Ok(SCAL_TO_OBJ_LONG.clone())
    } else if scalar_ty == &IdType::float() {
        Ok(SCAL_TO_OBJ_FLOAT.clone())
    } else if scalar_ty == &IdType::double() {
        Ok(SCAL_TO_OBJ_DOUBLE.clone())
    } else {
        bail!("{} is not a scalar", scalar_ty.__str__())
    }
}
