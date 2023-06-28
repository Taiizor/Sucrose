using System.IO;
using System.Reflection;
using ECT = Sucrose.Space.Enum.CommandsType;
using HC = Sucrose.Space.Helper.Command;
using R = Sucrose.Memory.Readonly;

namespace Sucrose.Tray.Command
{
    public static class Interface
    {
        public static void Command()
        {
#if TRAY_ICON_WPF
            string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

            HC.Run(Path.Combine(Folder, R.ConsoleApplication), $"{R.StartCommand}{ECT.Interface}{R.ValueSeparator}{Path.Combine(Folder, R.WPFApplication)}");
#elif TRAY_ICON_WinForms
            string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

            HC.Run(Path.Combine(Folder, R.ConsoleApplication), $"{R.StartCommand}{ECT.Interface}{R.ValueSeparator}{Path.Combine(Folder, R.WinFormsApplication)}");
#endif
        }
    }
}