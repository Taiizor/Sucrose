using SCHA = Sucrose.Commandog.Helper.Arguments;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SWDEMB = Sucrose.Watchdog.DarkErrorMessageBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using SWLEMB = Sucrose.Watchdog.LightErrorMessageBox;
using SWW = Sucrose.Watchdog.Watch;

namespace Sucrose.Commandog
{
    internal class App
    {
        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static bool HasError { get; set; } = true;

        [STAThread]
        internal static void Main(string[] Args)
        {
            try
            {
                SCHA.Parse(Args);
            }
            catch (Exception Exception)
            {
                SWW.Watch_CatchException(Exception);

                Message(Exception.Message);
            }
            finally
            {
                Close();
            }
        }

        protected static void Close()
        {
            Environment.Exit(0);
            Application.Exit();
        }

        protected static void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.CommandogLogManager.LogFile();

                switch (Theme)
                {
                    case SEWTT.Dark:
                        SWDEMB DarkMessageBox = new(Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SWLEMB LightMessageBox = new(Message, Path);
                        LightMessageBox.ShowDialog();
                        break;
                }

                Close();
            }
        }
    }
}