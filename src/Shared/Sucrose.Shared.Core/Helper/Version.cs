using SEAT = Skylark.Enum.AssemblyType;
using SHV = Skylark.Helper.Versionly;
using SWEOS = Skylark.Wing.Extension.OperatingSystem;
using SWHVI = Skylark.Wing.Helper.VersionInfo;
using SWPMV10P = Skylark.Wing.Provider.MajorVersion10Provider;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Version
    {
        public static string GetText()
        {
            return $"{Get()}";
        }

        public static string GetOSText()
        {
            return $"{GetOS()}";
        }

        public static System.Version Get()
        {
            return SHV.Auto(SEAT.Entry);
        }

        public static System.Version GetOS()
        {
            SWHVI Info = SWEOS.GetOSVersion();

            if (Info.Version.Major >= 10)
            {
                try
                {
                    SWPMV10P Provider = SWEOS.MajorVersion10Provider();

                    return new System.Version(Info.Version.Major, Info.Version.Minor, Info.Version.Build, Convert.ToInt32(Provider.UBR));
                }
                catch
                {
                    return Info.Version;
                }
            }
            else
            {
                return Info.Version;
            }
        }
    }
}