use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::error::{make_error_2037, make_error_2097};
pub use crate::avm2::object::file_reference_allocator;
use crate::avm2::object::{ByteArrayObject, FileReference};
use crate::avm2::{Activation, Avm2, Error, EventObject, Object, TObject, Value};
use crate::backend::ui::FileFilter;
use crate::string::AvmString;

pub fn get_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let bytearray = match *this.file_reference() {
        FileReference::FileDialogResult(ref dialog_result) if this.loaded() => {
            let bytes = dialog_result.contents();
            let storage = ByteArrayStorage::from_vec(bytes.to_vec());
            ByteArrayObject::from_storage(activation, storage)?
        }
        // Contrary to other getters `data` will return null instead of throwing.
        _ => return Ok(Value::Null),
    };

    Ok(bytearray.into())
}

pub fn get_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let name = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogResult(ref dialog_result) => {
            let name = dialog_result.file_name().unwrap_or_default();
            AvmString::new_utf8(activation.context.gc_context, name).into()
        }
    };

    Ok(name)
}

pub fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let size = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogResult(ref dialog_result) => dialog_result.size().unwrap_or(0),
    };

    Ok(Value::Number(size as f64))
}

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let mut filters = Vec::new();
    if let Value::Object(obj) = args[0] {
        if let Some(array_storage) = obj.as_array_storage() {
            for filter in array_storage.iter() {
                if let Some(Value::Object(obj)) = filter {
                    let filefilter = activation
                        .avm2()
                        .classes()
                        .filefilter
                        .inner_class_definition();
                    if !obj.is_of_type(filefilter, &mut activation.context) {
                        return Err(make_error_2097(activation));
                    }

                    let description = obj.get_public_property("description", activation)?;
                    let extension = obj.get_public_property("extension", activation)?;
                    let mac_type = obj.get_public_property("macType", activation)?;

                    // The description and extension must be non-empty strings.
                    match (description, extension) {
                        (Value::String(description), Value::String(extension))
                            if !description.is_empty() && !extension.is_empty() =>
                        {
                            let mac_type = match mac_type {
                                Value::String(mac_type) if !mac_type.is_empty() => {
                                    Some(mac_type.to_string())
                                }
                                _ => None,
                            };

                            filters.push(FileFilter {
                                description: description.to_string(),
                                extensions: extension.to_string(),
                                mac_type,
                            });
                        }
                        _ => return Err(make_error_2097(activation)),
                    }
                } else {
                    return Err(make_error_2097(activation));
                }
            }
        }
    }

    let dialog = activation.context.ui.display_file_open_dialog(filters);
    let result = match dialog {
        Some(dialog) => {
            let process = activation.context.load_manager.select_file_dialog_avm2(
                activation.context.player.clone(),
                this,
                dialog,
            );

            activation.context.navigator.spawn_future(process);
            true
        }
        None => false,
    };

    Ok(result.into())
}

pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    // Somewhat unexpectedly, we don't need to load anything here, because
    // that already happened during browse() or save().

    let size = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogResult(ref dialog_result) => dialog_result.size().unwrap_or(0),
    };

    let open_evt = EventObject::bare_default_event(&mut activation.context, "open");
    Avm2::dispatch_event(&mut activation.context, open_evt, this.into());

    let progress_evt = EventObject::progress_event(activation, "progress", 0, size, false, false);
    Avm2::dispatch_event(&mut activation.context, progress_evt, this.into());

    let open_evt2 = EventObject::bare_default_event(&mut activation.context, "open");
    Avm2::dispatch_event(&mut activation.context, open_evt2, this.into());

    let progress_evt2 =
        EventObject::progress_event(activation, "progress", size, size, false, false);
    Avm2::dispatch_event(&mut activation.context, progress_evt2, this.into());

    this.set_loaded(true);

    let complete_evt = EventObject::bare_default_event(&mut activation.context, "complete");
    Avm2::dispatch_event(&mut activation.context, complete_evt, this.into());

    Ok(Value::Undefined)
}
