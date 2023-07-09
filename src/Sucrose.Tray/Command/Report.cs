using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Tray.Command
{
    internal static class Report
    {
        public static void Command()
        {
            SSHC.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }
    }
}