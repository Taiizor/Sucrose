﻿using System.Diagnostics;

namespace Sucrose.Shared.Engine.Aurora.Manage
{
    internal static class Internal
    {
        public static string Application = string.Empty;

        public static Process ApplicationProcess = null;

        public static string ApplicationName = string.Empty;

        public static IntPtr ApplicationHandle = IntPtr.Zero;

        public static string ApplicationArguments = string.Empty;
    }
}