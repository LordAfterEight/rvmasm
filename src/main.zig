const std = @import("std");
const rvmasm = @import("rvmasm");
const loading = @import("load_src.zig");

var alloc = std.heap.c_allocator;

pub fn main() void {
    var src_path: []const u8 = "main.rvmasm";
    var out_path: []const u8 = "ROM.bin";
    var src_path_provided = false;

    var args = std.process.argsWithAllocator(alloc) catch return;
    defer args.deinit();
    _ = args.next();

    while (args.next()) |arg| {
        if (std.mem.eql(u8, arg, "-s")) {
            src_path = args.next() orelse {
                std.debug.print("\x1b[38;2;255;50;0mMissing file path\x1b[0m\n", .{});
                return;
            };
            src_path_provided = true;
        } else if (std.mem.eql(u8, arg, "-o")) {
            out_path = args.next() orelse "ROM.bin";
        } else {
            std.debug.print("\x1b[38;2;255;50;0mInvalid argument:\x1b[0m {s}\n", .{arg});
            return;
        }
    }
    if (!src_path_provided) {
        std.debug.print("\x1b[38;2;255;200;50mWARNING: No src path provided. Looking for '\x1b[38;2;255;50;255mmain.rvmasm\x1b[38;2;255;200;0m'\x1b[0m\n", .{});
    }


    // ======== Loading ========

    const buf = loading.load_src_file(alloc, src_path) catch {
        std.debug.print("\x1b[38;2;255;50;0mLoading failed\n", .{});
        return;
    };

    var line_counter: usize = 1;
    std.debug.print("\n \x1b[38;2;255;200;50m{d}\x1b[0m  ", .{line_counter});
    line_counter += 1;
    for (buf) |byte| {
        switch (byte) {
            '\n', '\r' => {
                std.debug.print("\n \x1b[38;2;255;200;50m{d}\x1b[0m  ", .{line_counter});
                line_counter += 1;
            },
            else => std.debug.print("{c}", .{byte}),
        }
    }
    std.debug.print("\n", .{});


    // ======== Parsing ========

    var lines = std.mem.tokenizeScalar(u8, buf, '\n');
    while (lines.next()) |line| {
        var tokens = std.mem.tokenizeScalar(u8, line, ' ');
        while (tokens.next()) |token| {
            std.debug.print("Token: {s}\n", .{token});
        }
        std.debug.print("\n", .{});
    }

    std.debug.print("{s}", .{out_path});
}
