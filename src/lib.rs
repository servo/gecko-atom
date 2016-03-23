/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate libc;
extern crate heapsize;

use std::borrow::Cow;
use std::char;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter;
use std::slice;
use heapsize::HeapSizeOf;

#[macro_export]
macro_rules! ns {
    () => { $crate::Namespace(atom!("")) };
    (html) => { $crate::Namespace(atom!("http://www.w3.org/1999/xhtml")) };
    (xml) => { $crate::Namespace(atom!("http://www.w3.org/XML/1998/namespace")) };
    (xmlns) => { $crate::Namespace(atom!("http://www.w3.org/2000/xmlns/")) };
    (xlink) => { $crate::Namespace(atom!("http://www.w3.org/1999/xlink")) };
    (svg) => { $crate::Namespace(atom!("http://www.w3.org/2000/svg")) };
    (mathml) => { $crate::Namespace(atom!("http://www.w3.org/1998/Math/MathML")) };
}

#[macro_export]
macro_rules! atom {
    ($s: expr) => { Atom::from($s) };
}

#[allow(non_camel_case_types)]
pub enum nsIAtom {}

#[derive(PartialEq, Eq)]
pub struct Atom(*mut nsIAtom);
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Namespace(pub Atom);

unsafe impl Send for Atom {}
unsafe impl Sync for Atom {}

extern {
    fn Gecko_NewAtom(s: *const libc::c_char, len: u32) -> *mut nsIAtom;
    fn Gecko_Atom_GetHash(atom: *mut nsIAtom) -> u32;
    fn Gecko_AddRefAtom(atom: *mut nsIAtom);
    fn Gecko_ReleaseAtom(atom: *mut nsIAtom);
    fn Gecko_Atom_GetUTF16String(atom: *mut nsIAtom, len: *mut u32) -> *const u16;
}

impl Atom {
    pub fn get_hash(&self) -> u32 {
        unsafe {
            Gecko_Atom_GetHash(self.0)
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe {
            let mut len = 0;
            let ptr = Gecko_Atom_GetUTF16String(self.0, &mut len);
            slice::from_raw_parts(ptr, len as usize)
        }
    }

    pub fn chars(&self) -> char::DecodeUtf16<iter::Cloned<slice::Iter<u16>>> {
        char::decode_utf16(self.as_slice().iter().cloned())
    }

    pub fn to_string(&self) -> String {
        String::from_utf16(self.as_slice()).unwrap()
    }

    pub fn as_ptr(&self) -> *mut nsIAtom {
        self.0
    }
}

impl Hash for Atom {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        state.write_u32(self.get_hash());
    }
}

impl Clone for Atom {
    #[inline(always)]
    fn clone(&self) -> Atom {
        unsafe {
            Gecko_AddRefAtom(self.0);
        }
        Atom(self.0)
    }
}

impl Drop for Atom {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            Gecko_ReleaseAtom(self.0);
        }
    }
}

impl HeapSizeOf for Atom {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl HeapSizeOf for Namespace {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Gecko Atom {:p}", self.0)
    }
}

impl PartialEq<str> for Atom {
    fn eq(&self, other: &str) -> bool {
        self.chars().zip(other.chars()).all(|(x, y)| x == Ok(y))
    }
}

impl PartialEq<Atom> for str {
    fn eq(&self, other: &Atom) -> bool {
        self.chars().zip(other.chars()).all(|(x, y)| Ok(x) == y)
    }
}

impl<'a> From<&'a str> for Atom {
    #[inline]
    fn from(string: &str) -> Atom {
        assert!(string.len() <= u32::max_value() as usize);
        Atom(unsafe {
            Gecko_NewAtom(string.as_ptr() as *const _, string.len() as u32)
        })
    }
}

impl<'a> From<Cow<'a, str>> for Atom {
    #[inline]
    fn from(string: Cow<'a, str>) -> Atom {
        Atom::from(&string[..])
    }
}

impl From<String> for Atom {
    #[inline]
    fn from(string: String) -> Atom {
        Atom::from(&string[..])
    }
}
