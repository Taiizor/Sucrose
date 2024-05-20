using Sucrose.Mpv.NET.API.Interop;
using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    [StructLayout(LayoutKind.Sequential)]
    public struct MpvEventProperty
    {
        public string Name;

        public MpvFormat Format;

        public IntPtr Data;

        public string DataString
        {
            get
            {
                if (Format == MpvFormat.None || Data == IntPtr.Zero)
                {
                    return default;
                }

                if (Format != MpvFormat.String)
                {
                    throw new MpvAPIException("Data is not a string.");
                }

                IntPtr innerPtr = Marshal.ReadIntPtr(Data);

                return MpvMarshal.GetManagedUTF8StringFromPtr(innerPtr);
            }
        }

        public long DataLong
        {
            get
            {
                if (Format == MpvFormat.None || Data == IntPtr.Zero)
                {
                    return default;
                }

                if (Format != MpvFormat.Int64)
                {
                    throw new MpvAPIException("Data is not a long.");
                }

                return Marshal.ReadInt64(Data);
            }
        }

        public double DataDouble
        {
            get
            {
                if (Format == MpvFormat.None || Data == IntPtr.Zero)
                {
                    return default;
                }

                if (Format != MpvFormat.Double)
                {
                    throw new MpvAPIException("Data is not a double.");
                }

                byte[] doubleBytes = new byte[sizeof(double)];
                Marshal.Copy(Data, doubleBytes, 0, sizeof(double));

                return BitConverter.ToDouble(doubleBytes, 0);
            }
        }
    }
}