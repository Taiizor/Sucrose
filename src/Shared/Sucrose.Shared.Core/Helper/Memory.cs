using System.Diagnostics;
using System.Management;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHN = Skylark.Helper.Numeric;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSSS = Skylark.Struct.Storage.StorageStruct;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Memory
    {
        public static long GetTotalMemory()
        {
            long TotalMemory = 0;

            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_OperatingSystem");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                TotalMemory = SSSHM.Check(Object, "TotalVisibleMemorySize", 0L);

                break;
            }

            double Result = SSESSE.Convert(TotalMemory, SEST.Kilobyte, SEST.Byte, SEMST.Palila);

            return Convert.ToInt64(SHN.Numeral(Result, false, false, 0, '0', SECNT.None));
        }

        public static string GetCurrentProcess()
        {
            Process CurrentProcess = Process.GetCurrentProcess();

            long UsedMemory = CurrentProcess.WorkingSet64;

            SSSSS Data = SSESSE.AutoConvert(UsedMemory, SEST.Byte, SEMST.Palila);

            return $"{SHN.Numeral(Data.Value, true, true, 1, '0', SECNT.None)} {Data.Short}";
        }
    }
}