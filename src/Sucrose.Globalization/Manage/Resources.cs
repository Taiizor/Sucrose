using Sucrose.Globalization.Strings;
using System.Globalization;
using System.Resources;

namespace Sucrose.Globalization.Manage
{
    public static class Resources
    {
        public static readonly CultureInfo CultureInfo = CultureInfo.CurrentUICulture;

        public static readonly ResourceManager TrayManager = new($"{typeof(Tray)}", typeof(Tray).Assembly);

        public static readonly ResourceManager GeneralManager = new($"{typeof(General)}", typeof(General).Assembly);
    }
}