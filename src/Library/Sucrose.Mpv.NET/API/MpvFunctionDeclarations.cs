using Sucrose.Mpv.NET.API.Interop;
using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    // Standard delegates
    // Taken from: https://github.com/mpv-player/mpv/blob/master/libmpv/client.h

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate long MpvClientAPIVersion();

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate long MpvClientId(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler))]
    public delegate string MpvErrorString(MpvError error);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void MpvFree(IntPtr data);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler))]
    public delegate string MpvClientName(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr MpvCreate();

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvInitialise(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void MpvDestroy(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void MpvTerminateDestroy(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr MpvCreateClient(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler))]
        out string name
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr MpvCreateWeakClient(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler))]
        out string name
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvLoadConfigFile(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string fileName
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate long MpvGetTimeUs(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvSetOption(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format,
        IntPtr data
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvSetOptionString(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string data
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvCommand(IntPtr mpvHandle, IntPtr args);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvCommandString(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string args
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvCommandAsync(IntPtr mpvHandle, ulong replyUserData, IntPtr args);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvAbortAsyncCommand(IntPtr mpvHandle, ulong replyUserData);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvSetProperty(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format,
        IntPtr data
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvSetPropertyString(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string data
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvSetPropertyAsync(
        IntPtr mpvHandle,
        ulong replyUserData,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format,
        IntPtr data
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvGetProperty(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format,
        out IntPtr data
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr MpvGetPropertyString(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr MpvGetPropertyOSDString(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvGetPropertyAsync(
        IntPtr mpvHandle,
        ulong replyUserData,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvObserveProperty(
        IntPtr mpvHandle,
        ulong replyUserData,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate int MpvUnobserveProperty(IntPtr mpvHandle, ulong registeredReplyUserData);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler))]
    public delegate string MpvEventName(MpvEventID eventID);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvRequestEvent(
        IntPtr mpvHandle,
        MpvEventID eventID,
        [MarshalAs(UnmanagedType.I1)]
        bool enable
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvRequestLogMessages(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string minLevel
    );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate IntPtr MpvWaitEvent(IntPtr mpvHandle, double timeout);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void MpvWakeup(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void MpvSetWakeupCallback(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.FunctionPtr)]
        MpvWakeupCallback cb,
        IntPtr d
     );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void MpvWaitAsyncRequests(IntPtr mpvHandle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvHookAdd(
        IntPtr mpvHandle,
        ulong replyUserData,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        int priority
     );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvHookContinue(IntPtr mpvHandle, ulong id);

    // Not strictly part of the C API but are used to invoke mpv_get_property with other value data types.

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvGetPropertyDouble(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format,
        out double data
     );

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate MpvError MpvGetPropertyLong(
        IntPtr mpvHandle,
        [MarshalAs(UnmanagedType.CustomMarshaler, MarshalTypeRef = typeof(MpvStringMarshaler), MarshalCookie = "free-com")]
        string name,
        MpvFormat format,
        out long data
     );

    // Other

    public delegate void MpvWakeupCallback(IntPtr d);
}