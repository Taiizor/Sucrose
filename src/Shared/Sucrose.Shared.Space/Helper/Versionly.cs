namespace Sucrose.Shared.Space.Helper
{
    internal static class Versionly
    {
        public static Version Increment(Version Version)
        {
            try
            {
                int Major = Version.Major;
                int Minor = Version.Minor;
                int Build = Version.Build;
                int Revision = Version.Revision + 1;

                if (Revision > 9)
                {
                    Revision = 0;
                    Build++;
                }

                if (Build > 9)
                {
                    Build = 0;
                    Minor++;
                }

                if (Minor > 9)
                {
                    Minor = 0;
                    Major++;
                }

                return new Version(Major, Minor, Build, Revision);
            }
            catch
            {
                return new Version(0, 0, 0, 0);
            }
        }

        public static Version Decrement(Version Version)
        {
            try
            {
                int Major = Version.Major;
                int Minor = Version.Minor;
                int Build = Version.Build;
                int Revision = Version.Revision - 1;

                if (Revision < 0)
                {
                    Revision = 9;
                    Build--;
                }

                if (Build < 0)
                {
                    Build = 9;
                    Minor--;
                }

                if (Minor < 0)
                {
                    Minor = 9;
                    Major--;
                }

                return new Version(Major, Minor, Build, Revision);
            }
            catch
            {
                return new Version(0, 0, 0, 0);
            }
        }
    }
}