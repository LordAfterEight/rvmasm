const std = @import("std");
const rvmasm = @import("rvmasm");
const loading = @import("load_src.zig");

var alloc = std.heap.c_allocator;


pub fn main() void {

    const src_path = "test.rvmasm";

    const buf = loading.load_file(alloc, src_path) catch {
        std.debug.print("\x1b[38;2;255;50;0mLoading failed\n", .{});
        return;
    };

    var line: usize = 1;
    std.debug.print("\n \x1b[38;2;255;200;50m{d}\x1b[0m  ", .{line});
    line += 1;
    for (buf) |byte| {
        switch (byte) {
            '\n', '\r' => {
                std.debug.print("\n \x1b[38;2;255;200;50m{d}\x1b[0m  ", .{line});
                line += 1;
            },
            else => std.debug.print("{c}", .{byte}),
        }
    }
}
