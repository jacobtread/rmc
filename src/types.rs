use std::os::raw::{c_double, c_float, c_int, c_uchar, c_uint, c_ushort};
use gl33::GLenum;


pub(crate) type GLboolean = c_uchar;
pub(crate) type GLbyte = i8;
pub(crate) type GLcharARB = u8;
pub(crate) type GLclampd = c_double;
pub(crate) type GLclampf = c_float;
pub(crate) type GLclampx = i32;
pub(crate) type GLdouble = c_double;
pub(crate) type GLfixed = i32;
pub(crate) type GLfloat = c_float;
pub(crate) type GLhalf = u16;
pub(crate) type GLhalfARB = u16;
pub(crate) type GLhalfNV = c_ushort;
pub(crate) type GLint = c_int;
pub(crate) type GLint64 = i64;
pub(crate) type GLint64EXT = i64;
pub(crate) type GLintptr = isize;
pub(crate) type GLintptrARB = isize;
pub(crate) type GLshort = i16;
pub(crate) type GLsizei = c_int;
pub(crate) type GLsizeiptr = isize;
pub(crate) type GLsizeiptrARB = isize;
pub(crate) type GLubyte = u8;
pub(crate) type GLuint = c_uint;
pub(crate) type GLuint64 = u64;
pub(crate) type GLuint64EXT = u64;
pub(crate) type GLushort = u16;
pub(crate) type GLvdpauSurfaceNV = GLintptr;
pub(crate) type GLvoid = void;
pub(crate) type void = core::ffi::c_void;

pub(crate) type GLchar = u8;
