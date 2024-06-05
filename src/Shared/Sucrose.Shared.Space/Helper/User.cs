using System.Management;
using System.Security.Principal;
using SHG = Skylark.Helper.Guidly;
using SMR = Sucrose.Memory.Readonly;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHU = Sucrose.Shared.Space.Helper.Unique;

namespace Sucrose.Shared.Space.Helper
{
    internal static class User
    {
        public static Guid GetGuid()
        {
            try
            {
                return SSSHU.Generate($"{GetName()}-{GetModel()}-{GetIdentifier()}");
            }
            catch
            {
                return SHG.TextToGuid(SMR.Guid);
            }
        }

        public static bool CheckGuid()
        {
            return !SMR.Guid.Equals(SHG.GuidToText(GetGuid()));
        }

        public static string GetName()
        {
            return Environment.UserName;
        }

        public static string GetUUID()
        {
            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystemProduct");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                return SSSHM.Check(Object, "UUID", string.Empty);
            }

            return string.Empty;
        }

        public static string GetModel()
        {
            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystem");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                return SSSHM.Check(Object, "Model", string.Empty);
            }

            return string.Empty;
        }

        public static string GetProcessor()
        {
            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                return SSSHM.Check(Object, "Name", string.Empty);
            }

            return string.Empty;
        }

        public static string GetIdentifier()
        {
            WindowsIdentity Identity = WindowsIdentity.GetCurrent();

            return Identity.User.Value;
        }

        public static int GetNumberOfCores()
        {
            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                return SSSHM.Check(Object, "NumberOfCores", 0);
            }

            return 0;
        }

        public static string GetIdentifying()
        {
            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystemProduct");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                return SSSHM.Check(Object, "IdentifyingNumber", string.Empty);
            }

            return string.Empty;
        }

        public static string GetProfilePath()
        {
            return Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        }

        public static string GetManufacturer()
        {
            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystem");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                return SSSHM.Check(Object, "Manufacturer", string.Empty);
            }

            return string.Empty;
        }
    }
}