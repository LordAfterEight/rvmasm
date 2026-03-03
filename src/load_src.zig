const std = @import("std");

const Memory = struct {
    data: std.ArrayList(u8) = .empty,
    mem_ptr: usize,
};

pub fn load_file(alloc: std.mem.Allocator, src_path: []const u8) anyerror![]u8 {
    const file = std.fs.cwd().openFile(src_path, .{ .mode = .read_only }) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not open file at path: {s}\nError:\x1b[0m {s}\n", .{src_path, @errorName(err)});
        return err;
    };
    _ = &file;

    var mem = alloc.create(Memory) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not create memory instance\nError:\x1b[0m {s}\n", .{@errorName(err)});
        return err;
    };
    mem.* = .{ .mem_ptr = 0 };

    var used_space: usize = 0;

    _ = &mem;
    _ = &used_space;

    const buf: []u8 = alloc.alloc(u8, 65536) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not initialize file buffer.\nError:\x1b[0m {s}\n", .{@errorName(err)});
        return err;
    };
    errdefer alloc.free(buf);

    const size = file.read(buf) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not read file\nError:\x1b[0m {s}\n", .{@errorName(err)});
        return err;
    };
    std.debug.print("\x1b[38;2;50;255;50mRead {d} Bytes\nSRC:\x1b[0m", .{size});
    return buf;
}
