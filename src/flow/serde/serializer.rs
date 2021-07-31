use serde::{ser, Serialize};
use super::error::{Error, Result};
use alloc::prelude::v1::{String, ToString};
use super::super::message::*;
use alloc::borrow::ToOwned;

pub struct Serializer {
    path: String,
}

impl Serializer {
    pub fn new(path: &str) -> Self {
        Serializer {
            path: path.to_owned(),
        }
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ArrSerializer;
    type SerializeTuple = ArrSerializer;
    type SerializeTupleStruct = ArrSerializer;
    type SerializeTupleVariant = ArrSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<()> {
        register!(content &self.path => BoolMessage (v));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        register!(content &self.path => I8Message (v));
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        register!(content &self.path => I16Message (v));
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        register!(content &self.path => I32Message (v));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        register!(content &self.path => I64Message (v));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        register!(content &self.path => U8Message (v));
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        register!(content &self.path => U16Message (v));
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        register!(content &self.path => U32Message (v));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        register!(content &self.path => U64Message (v));
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        register!(content &self.path => F32Message (v));
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        register!(content &self.path => F64Message (v));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        register!(content &self.path => CharMessage (v));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        register!(content &self.path => StringMessage (v));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        register!(content &self.path => ByteArrayMessage (v));
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<()> where T: Serialize {
        let mut ser = Serializer { path: format!("{}/{}", &self.path, variant) };
        value.serialize(&mut ser)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        let serializer = ArrSerializer { path: self.path.clone(), idx: 0 };
        Ok(serializer)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        let serializer = ArrSerializer { path: format!("{}/{}", &self.path, variant), idx: 0 };
        Ok(serializer)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        let serializer = MapSerializer { path: self.path.clone(), key: "".to_owned() };
        Ok(serializer)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        let serializer = MapSerializer { path: self.path.clone(), key: "".to_owned() };
        Ok(serializer)
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        let serializer = MapSerializer { path: format!("{}/{}", &self.path, variant), key: "".to_owned() };
        Ok(serializer)
    }
}

pub struct ArrSerializer {
    path: String,
    idx: usize,
}

impl <'a> ser::SerializeSeq for ArrSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", &self.path, self.idx) };
        value.serialize(&mut serializer)?;
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl <'a> ser::SerializeTuple for ArrSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", &self.path, self.idx) };
        value.serialize(&mut serializer)?;
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl <'a> ser::SerializeTupleStruct for ArrSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", &self.path, self.idx) };
        value.serialize(&mut serializer)?;
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl <'a> ser::SerializeTupleVariant for ArrSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", &self.path, self.idx) };
        value.serialize(&mut serializer)?;
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

struct StringSerializer {
    str: String,
}

impl <'a> ser::Serializer for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ArrSerializer;
    type SerializeTuple = ArrSerializer;
    type SerializeTupleStruct = ArrSerializer;
    type SerializeTupleVariant = ArrSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.str += &v.to_string();
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.str += v;
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<()> where T: Serialize {
        Err(Error::Unsupported)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::Unsupported)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::Unsupported)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        Err(Error::Unsupported)
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Unsupported)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Unsupported)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::Unsupported)
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        Err(Error::Unsupported)
    }
}

pub struct MapSerializer {
    path: String,
    key: String,
}

impl <'a> ser::SerializeStruct for MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", &self.path, key) };
        value.serialize(&mut serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl <'a> ser::SerializeStructVariant for MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", &self.path, key) };
        value.serialize(&mut serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl <'a> ser::SerializeMap for MapSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()> where T: Serialize {
        let mut serializer = StringSerializer { str: "".to_owned() };
        key.serialize(&mut serializer)?;
        self.key = serializer.str;
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()> where T: Serialize {
        let mut serializer = Serializer { path: format!("{}/{}", self.path, self.key) };
        value.serialize(&mut serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}