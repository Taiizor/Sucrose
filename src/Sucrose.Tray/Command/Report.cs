using System.IO;
using System.Reflection;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;

namespace Sucrose.Tray.Command
{
    public static class Report
    {
        public static void Command()
        {
            string Folder = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);

            SSHC.Run(Path.Combine(Folder, SMR.ConsoleApplication), $"{SMR.StartCommand}{SSECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }
    }
}