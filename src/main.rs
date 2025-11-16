#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate axlog;

extern crate alloc;
extern crate axruntime;

#[cfg(feature = "dyn")]
extern crate axdriver_dyn;

use alloc::{borrow::ToOwned, vec::Vec};

use axfs_ng::FS_CONTEXT;

mod entry;

pub const CMDLINE: &[&str] = &["/bin/sh", "-c", include_str!("init.sh")];

// pub const CMDLINE: &[&str] = &["/rknn_yolov8_demo/rknn_yolov8_demo", "/rknn_yolov8_demo/model/yolov8.rknn", "/rknn_yolov8_demo/model/bus.jpg"];
// pub const CMDLINE: &[&str] = &["/reverse/matmul_fp16", "1", "1024", "1024"];
// pub const CMDLINE: &[&str] = &["/reverse/matmul_4_36_16"];
// pub const CMDLINE: &[&str] = &["/reverse/matmul_int8", "1", "64", "64"];
// pub const CMDLINE: &[&str] = &["/reverse/matmul_fp16_fp16", "1", "768", "768"];
// pub const CMDLINE: &[&str] = &["/reverse/bench_mark", "2"];


#[unsafe(no_mangle)]
fn main() {
    starry_api::init();

    let args = CMDLINE
        .iter()
        .copied()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let envs = [];
    let exit_code = entry::run_initproc(&args, &envs);
    info!("Init process exited with code: {exit_code:?}");

    let cx = FS_CONTEXT.lock();
    cx.root_dir()
        .unmount_all()
        .expect("Failed to unmount all filesystems");
    cx.root_dir()
        .filesystem()
        .flush()
        .expect("Failed to flush rootfs");
}

#[cfg(feature = "vf2")]
extern crate axplat_riscv64_visionfive2;


#[cfg(target_arch = "aarch64")]
extern crate axplat_aarch64_dyn;
