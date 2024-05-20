using System.Runtime.InteropServices;
using System.Text;

namespace Sucrose.Mpv.NET.API.Interop
{
    public static class MpvMarshal
    {
        public static IntPtr GetComPtrFromManagedUTF8String(string @string)
        {
            Guard.AgainstNull(@string, nameof(@string));

            @string += '\0';

            byte[] stringBytes = Encoding.UTF8.GetBytes(@string);
            int stringBytesCount = stringBytes.Length;

            IntPtr stringPtr = Marshal.AllocCoTaskMem(stringBytesCount);
            Marshal.Copy(stringBytes, 0, stringPtr, stringBytesCount);

            return stringPtr;
        }

        public static string GetManagedUTF8StringFromPtr(IntPtr stringPtr)
        {
            if (stringPtr == IntPtr.Zero)
            {
                throw new ArgumentException("Cannot get string from invalid pointer.");
            }

            List<byte> stringBytes = new();
            int offset = 0;

            // Just to be safe!
            while (offset < short.MaxValue)
            {
                byte @byte = Marshal.ReadByte(stringPtr, offset);
                if (@byte == '\0')
                {
                    break;
                }

                stringBytes.Add(@byte);

                offset++;
            }

            byte[] stringBytesArray = stringBytes.ToArray();

            return Encoding.UTF8.GetString(stringBytesArray);
        }

        public static IntPtr GetComPtrForManagedUTF8StringArray(string[] inArray, out IntPtr[] outArray)
        {
            Guard.AgainstNull(inArray, nameof(inArray));

            int numberOfStrings = inArray.Length + 1;

            outArray = new IntPtr[numberOfStrings];

            // Allocate COM memory since this array will be passed to
            // a C function. This allocates space for the pointers that will point
            // to each string.
            IntPtr rootPointer = Marshal.AllocCoTaskMem(IntPtr.Size * numberOfStrings);

            for (int index = 0; index < inArray.Length; index++)
            {
                string currentString = inArray[index];
                IntPtr currentStringPtr = GetComPtrFromManagedUTF8String(currentString);

                outArray[index] = currentStringPtr;
            }

            Marshal.Copy(outArray, 0, rootPointer, numberOfStrings);

            return rootPointer;
        }

        public static void FreeComPtrArray(IntPtr[] ptrArray)
        {
            Guard.AgainstNull(ptrArray, nameof(ptrArray));

            foreach (IntPtr intPtr in ptrArray)
            {
                Marshal.FreeCoTaskMem(intPtr);
            }
        }

        public static TStruct PtrToStructure<TStruct>(IntPtr ptr) where TStruct : struct
        {
            if (ptr == IntPtr.Zero)
            {
                throw new ArgumentException("Invalid pointer.");
            }

            return (TStruct)Marshal.PtrToStructure(ptr, typeof(TStruct));
        }

        public static TDelegate LoadUnmanagedFunction<TDelegate>(IntPtr dllHandle, string functionName) where TDelegate : class
        {
            if (dllHandle == IntPtr.Zero)
            {
                throw new ArgumentException("DLL handle is invalid.", nameof(dllHandle));
            }

            Guard.AgainstNullOrEmptyOrWhiteSpaceString(functionName, nameof(functionName));

            IntPtr functionPtr = PlatformDll.Utils.GetProcAddress(dllHandle, functionName);
            if (functionPtr == IntPtr.Zero)
            {
                return null;
            }

            return (TDelegate)(object)Marshal.GetDelegateForFunctionPointer(functionPtr, typeof(TDelegate));
        }
    }
}