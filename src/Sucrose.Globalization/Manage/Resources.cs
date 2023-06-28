using System.Globalization;
using System.Resources;
using SGSG = Sucrose.Globalization.Strings.General;
using SGSMB = Sucrose.Globalization.Strings.MessageBox;
using SGST = Sucrose.Globalization.Strings.Tray;

namespace Sucrose.Globalization.Manage
{
    public static class Resources
    {
        public static CultureInfo CultureInfo = CultureInfo.CurrentUICulture;

        public static readonly ResourceManager TrayManager = new($"{typeof(SGST)}", typeof(SGST).Assembly);

        public static readonly ResourceManager GeneralManager = new($"{typeof(SGSG)}", typeof(SGSG).Assembly);

        public static readonly ResourceManager MessageBoxManager = new($"{typeof(SGSMB)}", typeof(SGSMB).Assembly);
    }
}