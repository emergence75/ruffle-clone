use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, ScriptObject, TObject, Value};
use crate::avm1_stub;
use crate::context::GcContext;

/// We store the connection state internally as part of our functional stub.
const ISCONNECTED_INTERNAL: &str = "_isConnected";

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection");
    this.define_value(
        activation.context.gc_context,
        ISCONNECTED_INTERNAL,
        Value::Bool(false),
        Attribute::DONT_ENUM | Attribute::DONT_DELETE,
    );
    Ok(this.into())
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "isConnected" => property(is_connected);
    "protocol" => property(protocol);
    "uri" => property(uri);

    "call" => method(call; DONT_ENUM | DONT_DELETE);
    "close" => method(close; DONT_ENUM | DONT_DELETE);
    "connect" => method(connect; DONT_ENUM | DONT_DELETE);
};

fn is_connected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection", "isConnected");
    this.get(ISCONNECTED_INTERNAL, activation)
}

fn protocol<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection", "protocol");
    Ok(Value::String("".into()))
}

fn uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection", "uri");
    Ok(Value::String("".into()))
}

fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection", "call");
    Ok(Value::Undefined)
}

fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection", "close");
    Ok(Value::Undefined)
}

fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetConnection", "connect");
    this.set(ISCONNECTED_INTERNAL, Value::Bool(true), activation)?;

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

pub fn create_class<'gc>(
    context: &mut GcContext<'_, 'gc>,
    netconnection_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        netconnection_proto,
    )
}
