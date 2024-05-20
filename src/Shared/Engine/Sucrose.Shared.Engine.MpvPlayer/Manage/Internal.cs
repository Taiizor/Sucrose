using MediaEngine = Sucrose.Mpv.NET.Player.MpvPlayer;

namespace Sucrose.Shared.Engine.MpvPlayer.Manage
{
    internal static class Internal
    {
        public static string Source;

        public static MediaEngine MediaEngine;

#if X86
        public static readonly string MediaPath = @"lib\libmpv-x86.dll";
#elif X64
        public static readonly string MediaPath = @"lib\libmpv-x64.dll";
#else
        public static readonly string MediaPath = @"lib\libmpv-ARM64.dll";
#endif

    }
}