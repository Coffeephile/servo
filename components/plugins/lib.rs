/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's compiler plugin/macro crate
//!
//! Attributes this crate provides:
//!
//!  - `#[privatize]` : Forces all fields in a struct/enum to be private
//!  - `#[jstraceable]` : Auto-derives an implementation of `JSTraceable` for a struct in the script crate
//!  - `#[must_root]` : Prevents data of the marked type from being used on the stack. See the lints module for more details
//!  - `#[dom_struct]` : Implies `#[privatize]`,`#[jstraceable]`, and `#[must_root]`.
//!     Use this for structs that correspond to a DOM type

#![feature(macro_rules, plugin_registrar, quote, phase)]

#![deny(unused_imports)]
#![deny(unused_variables)]

#[phase(plugin,link)]
extern crate syntax;
#[phase(plugin, link)]
extern crate rustc;
#[cfg(test)]
extern crate sync;

use rustc::lint::LintPassObject;
use rustc::plugin::Registry;
use syntax::ext::base::{Decorator, Modifier};

use syntax::parse::token::intern;

// Public for documentation to show up
/// Handles the auto-deriving for `#[jstraceable]`
pub mod jstraceable;
pub mod lints;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("dom_struct"), Modifier(box jstraceable::expand_dom_struct));
    reg.register_syntax_extension(intern("jstraceable"), Decorator(box jstraceable::expand_jstraceable));
    reg.register_lint_pass(box lints::TransmutePass as LintPassObject);
    reg.register_lint_pass(box lints::UnrootedPass as LintPassObject);
    reg.register_lint_pass(box lints::PrivatizePass as LintPassObject);
}


#[macro_export]
macro_rules! define_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident ),+,) => {
        define_css_keyword_enum!($name: $( $css => $variant ),+)
    };
    ($name: ident: $( $css: expr => $variant: ident ),+) => {
        #[allow(non_camel_case_types)]
        #[deriving(Clone, Eq, PartialEq, FromPrimitive)]
        pub enum $name {
            $( $variant ),+
        }

        impl $name {
            pub fn parse(component_value: &::cssparser::ast::ComponentValue) -> Result<$name, ()> {
                match component_value {
                    &::cssparser::ast::Ident(ref value) => {
                        match_ignore_ascii_case! { value:
                            $( $css => Ok($name::$variant) ),+
                            _ => Err(())
                        }
                    }
                    _ => Err(())
                }
            }
        }

        impl ::std::fmt::Show for $name {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use cssparser::ToCss;
                self.fmt_to_css(f)
            }
        }

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
            where W: ::text_writer::TextWriter {
                match self {
                    $( &$name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}


#[macro_export]
macro_rules! match_ignore_ascii_case {
    ( $value: expr: $( $string: expr => $result: expr ),+ _ => $fallback: expr, ) => {
        match_ignore_ascii_case! { $value:
            $( $string => $result ),+
            _ => $fallback
        }
    };
    ( $value: expr: $( $string: expr => $result: expr ),+ _ => $fallback: expr ) => {
        {
            use std::ascii::AsciiExt;
            match $value.as_slice() {
                $( s if s.eq_ignore_ascii_case($string) => $result, )+
                _ => $fallback
            }
        }
    };
}
