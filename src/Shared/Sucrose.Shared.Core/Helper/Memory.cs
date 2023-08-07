using System.Diagnostics;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHN = Skylark.Helper.Numeric;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSSSS = Skylark.Struct.Storage.StorageStruct;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Memory
    {

        public static string Get()
        {
            Process CurrentProcess = Process.GetCurrentProcess();
            long UsedMemory = CurrentProcess.WorkingSet64;

            SSSSS Data = SSESSE.AutoConvert(UsedMemory, SEST.Byte, SEMST.Palila);

            return $"{SHN.Numeral(Data.Value, true, true, 1, '0', SECNT.None)} {Data.Text}";
        }
    }
}