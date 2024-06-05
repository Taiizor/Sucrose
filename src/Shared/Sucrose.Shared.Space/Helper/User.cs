using System.Management;
using System.Security.Cryptography;
using System.Security.Principal;
using SEET = Skylark.Enum.EncodeType;
using SHE = Skylark.Helper.Encode;
using SHG = Skylark.Helper.Guidly;
using SMR = Sucrose.Memory.Readonly;
using SSSHM = Sucrose.Shared.Space.Helper.Management;

namespace Sucrose.Shared.Space.Helper
{
    internal static class User
    {
        public static Guid GetGuid()
        {
            try
            {
                return CreateGuid($"{GetName()}-{GetModel()}-{GetSecurityIdentifier()}");
            }
            catch
            {
                return SHG.TextToGuid(SMR.Guid);
            }
        }

        public static Guid NewGuid()
        {
            return Guid.NewGuid();
        }

        public static bool CheckGuid()
        {
            return !SMR.Guid.Equals(SHG.GuidToText(GetGuid()));
        }

        public static string GetName()
        {
            return Environment.UserName;
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

        public static string GetProfilePath()
        {
            return Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        }

        public static string GetSecurityIdentifier()
        {
            WindowsIdentity Identity = WindowsIdentity.GetCurrent();

            return Identity.User.Value;
        }

        private static Guid CreateGuid(string Value)
        {
            return SHG.ByteToGuid(MD5.Create().ComputeHash(SHE.GetBytes(Value, SEET.UTF8)));
        }
    }
}