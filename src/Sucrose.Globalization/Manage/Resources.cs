using System.Globalization;
using System.Resources;
using SGSG = Sucrose.Globalization.Strings.General;
using SGST = Sucrose.Globalization.Strings.Tray;
using SGSW = Sucrose.Globalization.Strings.Watchdog;

namespace Sucrose.Globalization.Manage
{
    public static class Resources
    {
        public static CultureInfo CultureInfo
        {
            get => CultureInfo.CurrentUICulture;
            set
            {
                CultureInfo.CurrentCulture = value;
                CultureInfo.CurrentUICulture = value;
                CultureInfo.DefaultThreadCurrentCulture = value;
                CultureInfo.DefaultThreadCurrentUICulture = value;
            }
        }

        public static readonly ResourceManager TrayManager = new($"{typeof(SGST)}", typeof(SGST).Assembly);

        public static readonly ResourceManager GeneralManager = new($"{typeof(SGSG)}", typeof(SGSG).Assembly);

        public static readonly ResourceManager WatchdogManager = new($"{typeof(SGSW)}", typeof(SGSW).Assembly);
    }
}