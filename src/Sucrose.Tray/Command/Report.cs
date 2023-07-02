using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SHA = Skylark.Helper.Assemblies;
using SEAT = Skylark.Enum.AssemblyType;

namespace Sucrose.Tray.Command
{
    public static class Report
    {
        public static void Command()
        {
            string Folder = Path.GetDirectoryName(SHA.Assemble(SEAT.Executing).Location);

            SSHC.Run(Path.Combine(Folder, SMR.ConsoleApplication), $"{SMR.StartCommand}{SSECT.Report}{SMR.ValueSeparator}{SMR.ReportWebsite}");
        }
    }
}