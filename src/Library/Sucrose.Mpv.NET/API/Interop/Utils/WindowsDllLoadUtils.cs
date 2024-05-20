using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API.Interop
{
    public class WindowsDllLoadUtils : IDllLoadUtils
    {
        // https://docs.microsoft.com/en-us/windows/desktop/api/libloaderapi/nf-libloaderapi-loadlibrarya
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Ansi, BestFitMapping = false)]
        public static extern IntPtr LoadLibrary(string lpFileName);

        // https://docs.microsoft.com/en-us/windows/desktop/api/libloaderapi/nf-libloaderapi-freelibrary
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Ansi, BestFitMapping = false)]
        public static extern int FreeLibrary(IntPtr hModule);

        // https://docs.microsoft.com/en-us/windows/desktop/api/libloaderapi/nf-libloaderapi-getprocaddress
        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Ansi, BestFitMapping = false)]
        public static extern IntPtr GetProcAddress(IntPtr hModule, string lProcName);

        IntPtr IDllLoadUtils.LoadLibrary(string fileName)
        {
            return LoadLibrary(fileName);
        }

        void IDllLoadUtils.FreeLibrary(IntPtr handle)
        {
            FreeLibrary(handle);
        }

        IntPtr IDllLoadUtils.GetProcAddress(IntPtr dllHandle, string name)
        {
            return GetProcAddress(dllHandle, name);
        }
    }
}