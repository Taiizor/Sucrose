using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Report
    {
        public static void Command()
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }
    }
}