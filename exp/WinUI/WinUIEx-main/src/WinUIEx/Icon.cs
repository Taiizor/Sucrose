using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Drawing;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using Windows.Win32;
using Windows.Foundation.Metadata;
using Windows.Win32.UI.WindowsAndMessaging;
using Windows.Win32.Foundation;

namespace WinUIEx
{
    /// <summary>
    /// Manages a native Windows Icon instance
    /// </summary>
    [CreateFromString(MethodName = "WinUIEx.Icon.FromFile")]
    public unsafe class Icon : IDisposable
    {
        private readonly HICON handle;
        private readonly Microsoft.Win32.SafeHandles.SafeFileHandle? _fileHandle;

        private Icon(Microsoft.Win32.SafeHandles.SafeFileHandle fileHandle) 
        {
            _fileHandle = fileHandle; //pin
            bool hInstanceAddRef = false;
            fileHandle.DangerousAddRef(ref hInstanceAddRef);
            handle = new HICON(fileHandle.DangerousGetHandle());
        }

        private Icon(HICON icon)
        {
            handle = icon;
        }

        internal HICON Handle => handle;

        /// <summary>
        /// Loads an icon from an .ico file.
        /// </summary>
        /// <param name="filename">Path to file</param>
        /// <returns>Icon</returns>
        public static Icon FromFile(string filename)
        {
            var handle = PInvoke.LoadImage(null, filename, GDI_IMAGE_TYPE.IMAGE_ICON, 16, 16, Windows.Win32.UI.Controls.IMAGE_FLAGS.LR_LOADFROMFILE);
            ThrowIfInvalid(handle);
            return new Icon(handle);
        }

        /// <summary>
        /// Creates an icon from a raw icon byte array
        /// </summary>
        /// <param name="rgba">RGBA byte array</param>
        /// <param name="size">The width and height of the image</param>
        /// <returns></returns>
        public static Icon FromByteArray(byte[] rgba, uint size)
        {
            byte[] ANDmaskIcon = new byte[size * size * 3];
            byte[] XORmaskIcon = new byte[size * size];
            for (int i = 0; i < size * size; i++)
            {
                ANDmaskIcon[i * 3] = rgba[i * 4+3];
                ANDmaskIcon[i * 3 + 1] = rgba[i * 4 + 2];
                ANDmaskIcon[i * 3 + 2] = rgba[i * 4 + 1];
                XORmaskIcon[i] = 0xAA;// rgba[i * 4 + 3];
            }

            var hinstance = PInvoke.GetModuleHandle((string?)null);
            HICON handle;
            fixed (byte* and = ANDmaskIcon)
            fixed (byte* xor = XORmaskIcon)
            {
                handle = PInvoke.CreateIcon(new HINSTANCE(hinstance.DangerousGetHandle()), 32, 32, 24, 1, xor, and);
            }
            ThrowIfInvalid(handle);
            return new Icon(handle);
        }

        private static void ThrowIfInvalid(HICON handle)
        {
            if (handle.Value == IntPtr.Zero)
            {
                var ex = new Win32Exception(System.Runtime.InteropServices.Marshal.GetLastWin32Error());
                throw ex;
            }
        }

        private static void ThrowIfInvalid(SafeHandle handle)
        {
            if (handle == null || handle.IsInvalid)
            {
                var ex = new Win32Exception(System.Runtime.InteropServices.Marshal.GetLastWin32Error());
                throw ex;
            }
        }

        /// <summary>
        /// For testing - Creates a simple Yang icon
        /// </summary>
        /// <returns>Icon with a Yang icon</returns>
        public static Icon Yang()
        {
            var hinstance = PInvoke.GetModuleHandle((string?)null);

            byte[] ANDmaskIcon = {0xFF, 0xFF, 0xFF, 0xFF,   // line 1 
                      0xFF, 0xFF, 0xC3, 0xFF,   // line 2 
                      0xFF, 0xFF, 0x00, 0xFF,   // line 3 
                      0xFF, 0xFE, 0x00, 0x7F,   // line 4 
 
                      0xFF, 0xFC, 0x00, 0x1F,   // line 5 
                      0xFF, 0xF8, 0x00, 0x0F,   // line 6 
                      0xFF, 0xF8, 0x00, 0x0F,   // line 7 
                      0xFF, 0xF0, 0x00, 0x07,   // line 8 
 
                      0xFF, 0xF0, 0x00, 0x03,   // line 9 
                      0xFF, 0xE0, 0x00, 0x03,   // line 10 
                      0xFF, 0xE0, 0x00, 0x01,   // line 11 
                      0xFF, 0xE0, 0x00, 0x01,   // line 12 
 
                      0xFF, 0xF0, 0x00, 0x01,   // line 13 
                      0xFF, 0xF0, 0x00, 0x00,   // line 14 
                      0xFF, 0xF8, 0x00, 0x00,   // line 15 
                      0xFF, 0xFC, 0x00, 0x00,   // line 16 
 
                      0xFF, 0xFF, 0x00, 0x00,   // line 17 
                      0xFF, 0xFF, 0x80, 0x00,   // line 18 
                      0xFF, 0xFF, 0xE0, 0x00,   // line 19 
                      0xFF, 0xFF, 0xE0, 0x01,   // line 20 
 
                      0xFF, 0xFF, 0xF0, 0x01,   // line 21 
                      0xFF, 0xFF, 0xF0, 0x01,   // line 22 
                      0xFF, 0xFF, 0xF0, 0x03,   // line 23 
                      0xFF, 0xFF, 0xE0, 0x03,   // line 24 
 
                      0xFF, 0xFF, 0xE0, 0x07,   // line 25 
                      0xFF, 0xFF, 0xC0, 0x0F,   // line 26 
                      0xFF, 0xFF, 0xC0, 0x0F,   // line 27 
                      0xFF, 0xFF, 0x80, 0x1F,   // line 28 
 
                      0xFF, 0xFF, 0x00, 0x7F,   // line 29 
                      0xFF, 0xFC, 0x00, 0xFF,   // line 30 
                      0xFF, 0xF8, 0x03, 0xFF,   // line 31 
                      0xFF, 0xFC, 0x3F, 0xFF};  // line 32 

            byte[] XORmaskIcon = {0x00, 0x00, 0x00, 0x00,   // line 1 
                      0x00, 0x00, 0x00, 0x00,   // line 2 
                      0x00, 0x00, 0x00, 0x00,   // line 3 
                      0x00, 0x00, 0x00, 0x00,   // line 4 
 
                      0x00, 0x00, 0x00, 0x00,   // line 5 
                      0x00, 0x00, 0x00, 0x00,   // line 6 
                      0x00, 0x00, 0x00, 0x00,   // line 7 
                      0x00, 0x00, 0x38, 0x00,   // line 8 
 
                      0x00, 0x00, 0x7C, 0x00,   // line 9 
                      0x00, 0x00, 0x7C, 0x00,   // line 10 
                      0x00, 0x00, 0x7C, 0x00,   // line 11 
                      0x00, 0x00, 0x38, 0x00,   // line 12 
 
                      0x00, 0x00, 0x00, 0x00,   // line 13 
                      0x00, 0x00, 0x00, 0x00,   // line 14 
                      0x00, 0x00, 0x00, 0x00,   // line 15 
                      0x00, 0x00, 0x00, 0x00,   // line 16 
 
                      0x00, 0x00, 0x00, 0x00,   // line 17 
                      0x00, 0x00, 0x00, 0x00,   // line 18 
                      0x00, 0x00, 0x00, 0x00,   // line 19 
                      0x00, 0x00, 0x00, 0x00,   // line 20 
 
                      0x00, 0x00, 0x00, 0x00,   // line 21 
                      0x00, 0x00, 0x00, 0x00,   // line 22 
                      0x00, 0x00, 0x00, 0x00,   // line 23 
                      0x00, 0x00, 0x00, 0x00,   // line 24 
 
                      0x00, 0x00, 0x00, 0x00,   // line 25 
                      0x00, 0x00, 0x00, 0x00,   // line 26 
                      0x00, 0x00, 0x00, 0x00,   // line 27 
                      0x00, 0x00, 0x00, 0x00,   // line 28 
 
                      0x00, 0x00, 0x00, 0x00,   // line 29 
                      0x00, 0x00, 0x00, 0x00,   // line 30 
                      0x00, 0x00, 0x00, 0x00,   // line 31 
                      0x00, 0x00, 0x00, 0x00};  // line 32 

            fixed (byte* and = ANDmaskIcon)
            fixed (byte* xor = XORmaskIcon)
            {
                var icon = PInvoke.CreateIcon(new HINSTANCE(hinstance.DangerousGetHandle()), 32, 32, 1, 1, and, xor);
                return new Icon(icon);
            }
        }

        /// <inheritdoc />
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        /// <inheritdoc />
        ~Icon()
        {
            Dispose(false);
        }

        /// <summary>
        /// Disposes the icon
        /// </summary>
        /// <param name="disposing"></param>
        protected void Dispose(bool disposing)
        {
            PInvoke.DestroyIcon(Handle); // also closes filehandle
        }
    }
}
