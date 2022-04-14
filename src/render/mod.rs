use core::panicking::panic;
use core::slice::SlicePattern;
use std::collections::HashMap;
use std::f32::NAN;
use std::io::Cursor;
use std::path::Prefix::Verbatim;
use std::ptr;
use std::ptr::eq;

use gl33::global_loader::{
    glDisableVertexAttribArray, glDrawElements, glEnableVertexAttribArray, glVertexAttribIPointer,
    glVertexAttribPointer,
};
use gl33::*;
use glutin::event::VirtualKeyCode::V;
use glutin::window::CursorIcon::VerticalText;
use ultraviolet::Vec3;
use crate::math::round_up_to_multiple;

use crate::types::{GLint, GLsizei, GLuint};

pub mod shader;
pub mod util;

#[derive(Debug, Copy, Clone)]
enum DataType {
    Float,
    UByte,
    Byte,
    UShort,
    Short,
    UInt,
    Int,
}

impl DataType {
    fn length(&self) -> usize {
        match self {
            DataType::UByte | DataType::Byte => 1,
            DataType::UShort | DataType::Short => 2,
            DataType::UInt | DataType::Int | DataType::Float => 4,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            DataType::Float => "float",
            DataType::UByte => "unsigned byte",
            DataType::Byte => "byte",
            DataType::UShort => "unsigned short",
            DataType::Short => "short",
            DataType::UInt => "unsigned int",
            DataType::Int => "int",
        }
    }

    fn gl(&self) -> VertexAttribPointerType {
        match self {
            DataType::Float => GL_FLOAT,
            DataType::UByte => GL_UNSIGNED_BYTE,
            DataType::Byte => GL_BYTE,
            DataType::UShort => GL_UNSIGNED_SHORT,
            DataType::Short => GL_SHORT,
            DataType::UInt => GL_UNSIGNED_INT,
            DataType::Int => GL_INT,
        }
    }
}

// enum VertexFormatElement {
//     Position,
//     Color,
//     Texture0,
//     Overlay,
//     Light,
//     Normal,
//     Padding,
// }

#[derive(Debug, Copy, Clone)]
enum ElementType {
    Position,
    Normal,
    Color,
    UV,
    Padding,
}

#[derive(Debug, Clone, Copy)]
struct VertexFormatElement {
    texture_index: u8,
    data_type: DataType,
    type_: ElementType,
    size: GLint,
}

impl VertexFormatElement {
    const POSITION: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Float, ElementType::Position, 3);
    const COLOR: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::UByte, ElementType::Color, 4);
    const TEXTURE: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Float, ElementType::UV, 2);
    const OVERLAY: &'static VertexFormatElement =
        &VertexFormatElement::new(1, DataType::Short, ElementType::UV, 2);
    const LIGHT: &'static VertexFormatElement =
        &VertexFormatElement::new(2, DataType::Short, ElementType::UV, 2);
    const NORMAL: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Byte, ElementType::Normal, 3);
    const PADDING: &'static VertexFormatElement =
        &VertexFormatElement::new(0, DataType::Byte, ElementType::Normal, 1);
}

impl VertexFormatElement {
    const fn new(
        texture_index: u8,
        data_type: DataType,
        type_: ElementType,
        size: GLint,
    ) -> VertexFormatElement {
        VertexFormatElement {
            texture_index,
            data_type,
            type_,
            size,
        }
    }

    unsafe fn start_drawing(&self, element_index: GLuint, stride: GLsizei, pointer: usize) {
        let data_type = &self.data_type;
        match self.type_ {
            ElementType::Position => {
                glEnableVertexAttribArray(element_index);
                glVertexAttribPointer(
                    element_index,
                    self.size,
                    data_type.gl(),
                    0,
                    stride,
                    pointer as *const _,
                );
            }
            ElementType::Normal | ElementType::Color => {
                glEnableVertexAttribArray(element_index);
                glVertexAttribPointer(
                    element_index,
                    self.size,
                    data_type.gl(),
                    1,
                    stride,
                    pointer as *const _,
                );
            }
            ElementType::UV => {
                glEnableVertexAttribArray(element_index);
                let gl_type = data_type.gl();
                if gl_type == GL_FLOAT {
                    glVertexAttribPointer(
                        element_index,
                        self.size,
                        gl_type,
                        0,
                        stride,
                        pointer as *const _,
                    );
                } else {
                    glVertexAttribIPointer(
                        element_index,
                        self.size,
                        gl_type,
                        stride,
                        pointer as *const _,
                    );
                }
            }
            ElementType::Padding => {}
        }
    }

    unsafe fn end_drawing(&self, element_index: GLuint) {
        match self.type_ {
            ElementType::Padding => {}
            _ => glDisableVertexAttribArray(element_index),
        }
    }

    fn byte_length(&self) -> usize {
        self.data_type.length() * self.size as usize
    }
}

#[derive(Debug)]
pub struct VertexFormat {
    pub attr_names: &'static [&'static str],
    pub elements: &'static [&'static VertexFormatElement],
    pub size: usize,
    pub offsets: Vec<usize>,
}

pub struct C(pub u8);

impl VertexFormat {
    pub const A: C = C(0);

    const fn new(
        names: &'static [&'static str],
        values: &'static [&'static VertexFormatElement],
    ) -> VertexFormat {
        let mut offsets = Vec::with_capacity(values.len());
        let mut size = 0;
        for x in values {
            offsets.push(size);
            size += x.byte_length();
        }
        VertexFormat {
            attr_names: names,
            elements: values,
            offsets,
            size,
        }
    }

    fn get_vertex_size_integer(&self) -> usize {
        self.size / 4
    }
}

impl VertexFormat {
    pub const BLIT_SCREEN: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV", "Color"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
        ],
    );
    const POSITION_COLOR_TEXTURE_LIGHT_NORMAL: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV0", "UV2", "Normal", "Padding"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::LIGHT,
            VertexFormatElement::NORMAL,
            VertexFormatElement::PADDING,
        ],
    );
    const POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL: &'static VertexFormat = &VertexFormat::new(
        &[
            "Position", "Color", "UV0", "UV1", "UV2", "Normal", "Padding",
        ],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::OVERLAY,
            VertexFormatElement::LIGHT,
            VertexFormatElement::NORMAL,
            VertexFormatElement::PADDING,
        ],
    );
    const POSITION_TEXTURE_COLOR_LIGHT: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "Color", "UV2"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
            VertexFormatElement::LIGHT,
        ],
    );
    pub const POSITION: &'static VertexFormat =
        &VertexFormat::new(&["Position"], &[VertexFormatElement::POSITION]);
    const POSITION_TEXTURE: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0"],
        &[VertexFormatElement::POSITION, VertexFormatElement::TEXTURE],
    );
    const POSITION_TEXTURE_COLOR: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "Color"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
        ],
    );
    const POSITION_COLOR_TEXTURE: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV0"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
        ],
    );
    const POSITION_COLOR: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color"],
        &[VertexFormatElement::POSITION, VertexFormatElement::COLOR],
    );
    const POSITION_COLOR_LIGHT: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV2"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::LIGHT,
        ],
    );
    const POSITION_COLOR_TEXTURE_LIGHT: &'static VertexFormat = &VertexFormat::new(
        &["Position", "Color", "UV0", "UV2"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::COLOR,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::LIGHT,
        ],
    );
    const POSITION_TEXTURE_LIGHT_COLOR: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "UV2", "Color"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::LIGHT,
            VertexFormatElement::COLOR,
        ],
    );
    const POSITION_TEXTURE_COLOR_NORMAL: &'static VertexFormat = &VertexFormat::new(
        &["Position", "UV0", "Color", "Normal", "Padding"],
        &[
            VertexFormatElement::POSITION,
            VertexFormatElement::TEXTURE,
            VertexFormatElement::COLOR,
            VertexFormatElement::NORMAL,
            VertexFormatElement::PADDING,
        ],
    );
}

struct BufferBuilder {
    buffer: Vec<u8>,
    build_start: usize,
    last_parameter_index: usize,
    next_draw_start: usize,
    current_element: Option<&'static VertexFormatElement>,
    current_element_id: usize,
    textures: bool,
    has_overlay: bool,
    building: bool,
    camera_offset: bool,
    element_offset: usize,
    vertex_count: usize,
    format: Option<&'static VertexFormat>,
    draw_mode: Option<DrawMode>,
    current_parameters: Option<Vec<Vec3>>,
    camera: Vec3,
    parameters: Vec<DrawArrayParameters>,
}

impl BufferBuilder {
    pub fn new(initial_capacity: usize) -> BufferBuilder {
        BufferBuilder {
            buffer: Vec::with_capacity(initial_capacity * 6),
            build_start: 0,
            last_parameter_index: 0,
            next_draw_start: 0,
            current_element: None,
            current_element_id: 0,
            textures: false,
            has_overlay: false,
            building: false,
            camera_offset: false,
            element_offset: 0,
            vertex_count: 0,
            format: None,
            draw_mode: None,
            current_parameters: None,
            camera: Vec3::new(f32::NAN, f32::NAN, f32::NAN),
            parameters: Vec::new(),
        }
    }

    fn put_byte(&mut self, value: u8) {
        self.buffer.push(value);
    }

    fn put_short(&mut self, value: u16) {
        let v: [u8; 2] = u16::to_be_bytes(value);
        self.buffer.push(v[0]);
        self.buffer.push(v[1]);
    }

    fn put_float(&mut self,  value: f32) {
        let v: [u8; 4] = f32::to_be_bytes(value);
        self.buffer.push(v[0]);
        self.buffer.push(v[1]);
        self.buffer.push(v[2]);
        self.buffer.push(v[3]);
    }

    fn get_float(&self, index: usize) -> f32 {
        let bytes = self.buffer[index..(index + 4)] as [u8; 4];
        f32::from_be_bytes(bytes)
    }

    pub fn set_camera_position(&mut self, camera: Vec3) {
        if self.draw_mode.is_some() {
            if let DrawMode::Quads = self.draw_mode.unwrap() {
                return;
            }
            self.camera = camera;
            if self.current_parameters.is_none() {
                self.current_parameters = Some(self.get_parameter_vec());
            }
        }
    }

    pub fn get_parameter_vec(&self) -> Vec<Vec3> {
        let i = self.build_start / 4;
        let j = self.format.unwrap().get_vertex_size_integer();
        let dms = self.draw_mode.unwrap().size();
        let k = j * dms;
        let l = self.vertex_count / dms;
        let mut params = Vec::with_capacity(l);
        for m in 0..l {
            let f = self.get_float(i + m * k);
            let g = self.get_float(i + m * k + 1);
            let h = self.get_float(i + m * k + 2);
            let n = self.get_float(i + m * k + j * 2);
            let o = self.get_float(i + m * k + j * 2 + 1);
            let p = self.get_float(i + m * k + j * 2 + 2);
            let q = (f + n) / 2.0;
            let r = (g + o) / 2.0;
            let s = (h + p) / 2.0;
            params.push(Vec3::new(q, r, s))
        }
        params
    }

    pub fn pop_state(&self) -> State {
        State {
            draw_mode: self.draw_mode,
            vertex_count: self.vertex_count,
            params: self.current_parameters.clone(),
            camera: self.camera.clone(),
        }
    }

    fn clear_buffer(&mut self) {
        self.buffer.fill(0);
    }

    pub fn restore_state(&mut self, state: State) {
        self.clear_buffer();
        self.draw_mode = state.draw_mode;
        self.vertex_count = state.vertex_count;
        self.element_offset = self.build_start;
        self.current_parameters = state.params;
        self.camera = state.camera;
        self.camera_offset = true;
    }

    pub fn begin(&mut self, mode: DrawMode, format: &'static VertexFormat) {
        if self.building {
            panic!("already building buffer cannot build twice");
        }
        self.building = true;
        self.draw_mode = Some(mode);
        self.set_format(format);
        self.current_element = Some(format.elements[0]);
        self.current_element_id = 0;
        self.clear_buffer();
    }

    fn set_format(&mut self, format: &'static VertexFormat) {
        if self.format.is_none() || eq(self.format.unwrap(), format) {
            return;
        }
        self.format = Some(format);
        let bl = eq(
            format,
            VertexFormat::POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL,
        );
        let bl2 = eq(format, VertexFormat::POSITION_COLOR_TEXTURE_LIGHT_NORMAL);
        self.textures = bl || bl2;
        self.has_overlay = bl;
    }

    pub fn reset(&mut self) {
        self.build_start = 0;
        self.next_draw_start = 0;
        self.element_offset = 0;
        self.current_parameters = None;
        self.last_parameter_index = 0;
    }

    pub fn pop_data(&mut self) -> (DrawArrayParameters, Vec<u8>)  {
        let param = self.parameters[self.last_parameter_index];
        let start = self.next_draw_start;
        self.next_draw_start += round_up_to_multiple(param.get_draw_start() as i32, 4) as usize;
        let end = self.next_draw_start;
        if self.last_parameter_index == self.parameters.len() && self.vertex_count == 0 {
            self.clear_buffer()
        }
        let values = self.buffer[start..end].to_vec();
        (param, values)
    }

    pub fn vertex(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32, a: f32)
}

#[derive(Clone, Copy)]
struct DrawArrayParameters {
    vertex_format: &'static VertexFormat,
    count: usize,
    vertex_count: usize,
    draw_mode: DrawMode,
    element_format: IntType,
    camera_offset: bool,
    textured: bool,
}

impl DrawArrayParameters {
    pub fn get_limit(&self) -> usize {
        self.count * self.vertex_format.size
    }

    pub fn get_draw_length(&self) -> usize {
        if self.textured {
            0
        } else {
            self.vertex_count * self.element_format.size()
        }
    }

    pub fn get_draw_start(&self) -> usize {
        self.get_limit() + self.get_draw_length()
    }
}

struct State {
    draw_mode: Option<DrawMode>,
    vertex_count: usize,
    params: Option<Vec<Vec3>>,
    camera: Vec3,
}

#[derive(Copy, Clone)]
enum DrawMode {
    Lines,
    LineStrip,
    DebugLines,
    DebugLineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
    Quads,
}

impl DrawMode {
    pub fn size(&self) -> usize {
        match self {
            DrawMode::LineStrip
            | DrawMode::DebugLineStrip
            | DrawMode::TriangleStrip
            | DrawMode::TriangleFan => 1,
            DrawMode::Lines | DrawMode::DebugLines => 2,
            DrawMode::Triangles => 3,
            DrawMode::Quads => 4,
        }
    }

    pub fn get_size(&self, vertex_count: usize) -> usize {
        match self {
            DrawMode::Lines | DrawMode::Quads => vertex_count,
            _ => vertex_count / 4 * 6,
        }
    }
}

impl Into<PrimitiveType> for DrawMode {
    fn into(&self) -> PrimitiveType {
        match self {
            DrawMode::Lines | DrawMode::Triangles => GL_TRIANGLES,
            DrawMode::LineStrip | DrawMode::TriangleStrip | DrawMode::Quads => GL_TRIANGLE_STRIP,
            DrawMode::DebugLines => GL_LINE_STRIP,
            DrawMode::DebugLineStrip => GL_LINES,
            DrawMode::TriangleFan => GL_TRIANGLE_FAN,
        }
    }
}

#[derive(Clone, Copy)]
enum IntType {
    Byte,
    Short,
    Int,
}

impl Into<VertexAttribIType> for IntType {
    fn into(&self) -> VertexAttribIType {
        match self {
            IntType::Byte => GL_UNSIGNED_BYTE,
            IntType::Short => GL_UNSIGNED_SHORT,
            IntType::Int => GL_UNSIGNED_INT,
        }
    }
}

impl IntType {
    pub fn size(&self) -> usize {
        match self {
            IntType::Byte => 1,
            IntType::Short => 2,
            IntType::Int => 4,
        }
    }

    pub fn get_smallest_for(value: usize) -> IntType {
        if (value & -0x10000) != 0 {
            IntType::Int
        } else if (value & 0xFF00) != 0 {
            IntType::Short
        } else {
            IntType::Byte
        }
    }
}
