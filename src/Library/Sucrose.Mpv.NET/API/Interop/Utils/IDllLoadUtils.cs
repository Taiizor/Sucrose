namespace Sucrose.Mpv.NET.API.Interop
{
    public interface IDllLoadUtils
    {
        IntPtr LoadLibrary(string fileName);

        void FreeLibrary(IntPtr handle);

        IntPtr GetProcAddress(IntPtr dllHandle, string name);
    }
}