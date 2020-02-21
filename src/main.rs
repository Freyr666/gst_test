extern crate libc;
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
extern crate gstreamer as gst;
extern crate gstreamer_sys as gst_sys;

use std::ffi::{CString, CStr};
use gst::*;
use glib::*;
use glib::translate::ToGlibPtr;
use std::sync::{Arc, Mutex};

struct Context {
    pipeline : gst::Pipeline,
}

unsafe fn string_to_chars (s: &str) -> *mut libc::c_char {
    let size = s.len();    
    //let cstr = CString::new(res).unwrap();
    let buf = libc::calloc(size + 1, 1);
    libc::memcpy(buf, s.as_ptr() as *const libc::c_void, size);
    buf as *mut libc::c_char
}

unsafe extern "C" fn on_destroy (p : glib_sys::gpointer) {
    let c = CStr::from_ptr(p as *mut libc::c_char);
    println!("Element {} was destroyed", c.to_str().unwrap());
}

impl Context {
    fn new () -> Context {
        let pipeline = gst::Pipeline::new (None);
        let videosrc = gst::ElementFactory::make("videotestsrc", None).unwrap();
        let videosink = gst::ElementFactory::make("autovideosink", None).unwrap();
        pipeline.add_many(&[&videosrc, &videosink]).unwrap();
        gst::Element::link_many(&[&videosrc, &videosink]).unwrap();

        let stor_name = CString::new("destroy").unwrap();
        unsafe {
            let name = string_to_chars(&videosrc.get_name().as_str());
            let c : *mut gst_sys::GstElement = videosrc.to_glib_full();
            gobject_sys::g_object_set_data_full(c as *mut gobject_sys::GObject,
                                                stor_name.as_ptr() as *const libc::c_char,
                                                name as glib_sys::gpointer,
                                                Some(on_destroy));
        }
        unsafe {
            let name = string_to_chars(&videosink.get_name().as_str());
            let c : *mut gst_sys::GstElement = videosink.to_glib_full();
            gobject_sys::g_object_set_data_full(c as *mut gobject_sys::GObject,
                                                stor_name.as_ptr() as *const libc::c_char,
                                                name as glib_sys::gpointer,
                                                Some(on_destroy));
        }
        unsafe {
            let name = string_to_chars(&pipeline.get_name().as_str());
            let c : *mut gst_sys::GstPipeline = pipeline.to_glib_full();
            gobject_sys::g_object_set_data_full(c as *mut gobject_sys::GObject,
                                                stor_name.as_ptr() as *const libc::c_char,
                                                name as glib_sys::gpointer,
                                                Some(on_destroy));
        }
        pipeline.set_state(gst::State::Playing).unwrap();
        println!("Pipeline refcounter {}", pipeline.ref_count());
        // Refcounter equals two here
        Context { pipeline }
    }

    fn reset (&mut self) {
        self.pipeline.set_state(gst::State::Null).unwrap();
        println!("Pipeline refcounter on reset {}", self.pipeline.ref_count());
        // unsafe { // Refcounter equals two for some reason
        //    let p : *mut gst_sys::GstPipeline = self.pipeline.to_glib_full();
        //    gstreamer_sys::gst_object_unref(p as *mut gst_sys::GstObject);
        //    gstreamer_sys::gst_object_unref(p as *mut gst_sys::GstObject);
        //}
        self.pipeline = gst::Pipeline::new (None);
    }

}

fn main () {
    gst::init();
    let mainloop = glib::MainLoop::new (None, false);

    let context = Arc::new(Mutex::new(Context::new()));
    glib::timeout_add_seconds(3, move || {
        context.lock().unwrap().reset();
        glib::Continue(false)
    });
    
    mainloop.run ();
}
