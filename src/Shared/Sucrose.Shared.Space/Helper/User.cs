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

        public static string GetName()
        {
            try
            {
                return Environment.UserName;
            }
            catch
            {
                return SMR.Default;
            }
        }

        public static string GetUUID()
        {
            try
            {
                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystemProduct");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    return SSSHM.Check(Object, "UUID", SMR.Default);
                }

                return SMR.Default;
            }
            catch
            {
                return SMR.Default;
            }
        }

        public static int GetLanguage()
        {
            try
            {
                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_OperatingSystem");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    return SSSHM.Check(Object, "OSLanguage", 0);
                }

                return 0;
            }
            catch
            {
                return 0;
            }
        }

        public static string GetModel()
        {
            try
            {
                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystem");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    return SSSHM.Check(Object, "Model", SMR.Default);
                }

                return SMR.Default;
            }
            catch
            {
                return SMR.Default;
            }
        }

        public static string[] GetGraphic()
        {
            try
            {
                List<string> Names = new();

                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_VideoController");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    Names.Add(SSSHM.Check(Object, "Name", SMR.Default));
                }

                return Names.ToArray();
            }
            catch
            {
                return new[] { SMR.Default };
            }
        }

        public static string[] GetNetwork()
        {
            try
            {
                List<string> Names = new();

                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_NetworkAdapter");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    Names.Add(SSSHM.Check(Object, "Name", SMR.Default));
                }

                return Names.ToArray();
            }
            catch
            {
                return new[] { SMR.Default };
            }
        }

        public static string GetIdentifier()
        {
            try
            {
                WindowsIdentity Identity = WindowsIdentity.GetCurrent();

                return Identity.User.Value;
            }
            catch
            {
                return SMR.Default;
            }
        }

        public static int GetNumberOfCores()
        {
            try
            {
                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    return SSSHM.Check(Object, "NumberOfCores", 0);
                }

                return 0;
            }
            catch
            {
                return 0;
            }
        }

        public static string GetIdentifying()
        {
            try
            {
                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystemProduct");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    return SSSHM.Check(Object, "IdentifyingNumber", SMR.Default);
                }

                return SMR.Default;
            }
            catch
            {
                return SMR.Default;
            }
        }

        public static string[] GetProcessor()
        {
            try
            {
                List<string> Names = new();

                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_Processor");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    Names.Add(SSSHM.Check(Object, "Name", SMR.Default));
                }

                return Names.ToArray();
            }
            catch
            {
                return new[] { SMR.Default };
            }
        }

        public static string GetProfilePath()
        {
            try
            {
                return Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            }
            catch
            {
                return SMR.Default;
            }
        }

        public static string GetManufacturer()
        {
            try
            {
                ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_ComputerSystem");

                foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
                {
                    return SSSHM.Check(Object, "Manufacturer", SMR.Default);
                }

                return SMR.Default;
            }
            catch
            {
                return SMR.Default;
            }
        }
    }
}