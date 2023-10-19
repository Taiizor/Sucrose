using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Watchdog
    {
        public static void Start(string Message, string Path)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Watchdog}{SMR.ValueSeparator}{SSSMI.Watchdog}{SMR.ValueSeparator}{Message}{SMR.ValueSeparator}{Path}");
        }

        public static void Start(string Message, string Path, string Source, string Text)
        {
            if (string.IsNullOrEmpty(Source) || string.IsNullOrEmpty(Text))
            {
                Start(Message, Path);
            }
            else
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Watchdog}{SMR.ValueSeparator}{SSSMI.Watchdog}{SMR.ValueSeparator}{Message}{SMR.ValueSeparator}{Path}{SMR.ValueSeparator}{Source}{SMR.ValueSeparator}{Text}");
            }
        }
    }
}