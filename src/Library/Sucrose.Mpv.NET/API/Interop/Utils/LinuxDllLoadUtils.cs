using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API.Interop
{
    public class LinuxDllLoadUtils : IDllLoadUtils
    {
        [DllImport("libc")]
        private static extern IntPtr dlopen(string fileName, int flags);

        [DllImport("libc")]
        private static extern IntPtr dlsym(IntPtr handle, string symbol);

        [DllImport("libc")]
        private static extern int dlclose(IntPtr handle);

        [DllImport("libc")]
        private static extern IntPtr dlerror();

        private const int RTLD_NOW = 2;

        public IntPtr LoadLibrary(string fileName)
        {
            return dlopen(fileName, RTLD_NOW);
        }

        public void FreeLibrary(IntPtr handle)
        {
            dlclose(handle);
        }

        public IntPtr GetProcAddress(IntPtr dllHandle, string name)
        {
            // Clear previous errors if any.
            dlerror();

            IntPtr res = dlsym(dllHandle, name);

            IntPtr errPtr = dlerror();
            if (errPtr != IntPtr.Zero)
            {
                throw new MpvAPIException("dlsym: " + Marshal.PtrToStringAnsi(errPtr));
            }

            return res;
        }
    }
}