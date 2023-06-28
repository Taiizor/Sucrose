using System.IO;
using System.Reflection;
using ECT = Sucrose.Space.Enum.CommandsType;
using HC = Sucrose.Space.Helper.Command;
using R = Sucrose.Memory.Readonly;

namespace Sucrose.Tray.Command
{
    public static class Report
    {
        public static void Command()
        {
            string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

            HC.Run(Path.Combine(Folder, R.ConsoleApplication), $"{R.StartCommand}{ECT.Report}{R.ValueSeparator}{R.ReportWebsite}");
        }
    }
}