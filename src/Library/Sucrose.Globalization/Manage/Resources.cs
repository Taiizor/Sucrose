using System.Resources;
using SGSC = Sucrose.Globalization.Strings.Commandog;
using SGSD = Sucrose.Globalization.Strings.Discord;
using SGSG = Sucrose.Globalization.Strings.General;
using SGSL = Sucrose.Globalization.Strings.Launcher;

namespace Sucrose.Globalization.Manage
{
    public static class Resources
    {
        public static readonly ResourceManager DiscordManager = new($"{typeof(SGSD)}", typeof(SGSD).Assembly);

        public static readonly ResourceManager GeneralManager = new($"{typeof(SGSG)}", typeof(SGSG).Assembly);

        public static readonly ResourceManager LauncherManager = new($"{typeof(SGSL)}", typeof(SGSL).Assembly);

        public static readonly ResourceManager CommandogManager = new($"{typeof(SGSC)}", typeof(SGSC).Assembly);
    }
}