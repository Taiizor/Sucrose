using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Reportdog
    {
        public static void Command()
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Reportdog}{SMMRG.ValueSeparator}{SSSMI.Reportdog}");
        }
    }
}