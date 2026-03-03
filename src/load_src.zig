const std = @import("std");

const Memory = struct {
    data: std.ArrayList(u8) = .empty,
    mem_ptr: usize,
};

/// Loads a file using a relative path.
/// Returns a slice []u8 containing the raw bytes of the file
///
/// Can return various errors depending on whether the file cannot
/// be opened or read or the file buffer or memory instance could
/// not be initialized.
pub fn load_src_file(alloc: std.mem.Allocator, src_path: []const u8) anyerror![]u8 {
    const file = std.fs.cwd().openFile(src_path, .{ .mode = .read_only }) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not open file at path: {s}\nError:\x1b[0m {s}\n", .{ src_path, @errorName(err) });
        return err;
    };
    _ = &file;

    const mem = alloc.create(Memory) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not create memory instance\nError:\x1b[0m {s}\n", .{@errorName(err)});
        return err;
    };
    mem.* = .{ .mem_ptr = 0 };

    const buf: []u8 = alloc.alloc(u8, 65536) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not initialize file buffer.\nError:\x1b[0m {s}\n", .{@errorName(err)});
        return err;
    };
    errdefer alloc.free(buf);

    const size = file.read(buf) catch |err| {
        std.debug.print("\x1b[38;2;255;50;0mCould not read file\nError:\x1b[0m {s}\n", .{@errorName(err)});
        return err;
    };

    if (size < 1024) {
        std.debug.print("\x1b[38;2;50;255;50mRead {d} Bytes\nSRC:\x1b[0m", .{size});
    } else if (size < 1024 * 1024) {
        std.debug.print("\x1b[38;2;50;255;50mRead {d}KiB\nSRC:\x1b[0m", .{size / 1024});
    } else if (size < 1024 * 1024 * 1024) {
        std.debug.print("\x1b[38;2;50;255;50mRead {d}MiB\nSRC:\x1b[0m", .{size / 1024 / 1024});
    }
    return buf;
}
