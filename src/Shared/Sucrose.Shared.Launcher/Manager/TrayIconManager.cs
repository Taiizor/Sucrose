using System.Globalization;
using System.IO;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSLCC = Sucrose.Shared.Launcher.Command.Close;
using SSLCE = Sucrose.Shared.Launcher.Command.Engine;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLCR = Sucrose.Shared.Launcher.Command.Report;
using SSLCS = Sucrose.Shared.Launcher.Command.Setting;
using SSLCU = Sucrose.Shared.Launcher.Command.Update;
using SSLHC = Sucrose.Shared.Launcher.Helper.Calculate;
using SSLHR = Sucrose.Shared.Launcher.Helper.Radius;
using SSLRDR = Sucrose.Shared.Launcher.Renderer.DarkRenderer;
using SSLRLR = Sucrose.Shared.Launcher.Renderer.LightRenderer;
using SSLSSS = Sucrose.Shared.Launcher.Separator.StripSeparator;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHA = Sucrose.Shared.Space.Helper.Assets;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Launcher.Manager
{
    public class TrayIconManager
    {
        private static string LibraryLocation => SMMI.EngineSettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SHC.CurrentUITwoLetterISOLanguageName);

        private static string LibrarySelected => SMMI.EngineSettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static bool Visible => SMMI.LauncherSettingManager.GetSetting(SMC.Visible, true);

        private ContextMenuStrip ContextMenu { get; set; } = new();

        private NotifyIcon TrayIcon { get; set; } = new();

        public void Start()
        {
            TrayIcon.Text = SSRER.GetValue("Launcher", "TrayText");
            TrayIcon.Icon = new Icon(SSSHA.Get(SSRER.GetValue("Launcher", "TrayIcon")));

            TrayIcon.MouseClick += MouseClick;
            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseDoubleClick += MouseDoubleClick;

            TrayIcon.Visible = Visible;

            ContextMenuAdjustment();

            SSLCE.Command(false);
        }

        public void Initialize()
        {
            SSRHR.SetLanguage(Culture);
            SHC.All = new CultureInfo(Culture, true);

            SSLHR.Corner(ContextMenu);

            if (Theme == SEWTT.Dark)
            {
                ContextMenu.Renderer = new SSLRDR();
            }
            else
            {
                ContextMenu.Renderer = new SSLRLR();
            }

            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "OpenText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "OpenIcon"))), CommandInterface);

            SSLSSS Separator1 = new(Theme);

            if (SSSHL.Run())
            {
                ContextMenu.Items.Add(Separator1.Strip);

                ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallCloseText"), null, CommandEngine);
                //ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallStartText"), null, null); //WallStopText

                //ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallChangeText"), null, null);

                string PropertiesPath = Path.Combine(LibraryLocation, LibrarySelected, SMR.SucroseProperties);

                if (File.Exists(PropertiesPath))
                {
                    ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallCustomizeText"), null, null);
                }
            }
            else if (SMMI.EngineSettingManager.CheckFile())
            {
                string InfoPath = Path.Combine(LibraryLocation, LibrarySelected, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    ContextMenu.Items.Add(Separator1.Strip);

                    ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallOpenText"), null, CommandEngine);
                }
            }

            SSLSSS Separator2 = new(Theme);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "SettingText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "SettingIcon"))), CommandSetting);
            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "ReportText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "ReportIcon"))), CommandReport);
            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "UpdateText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "UpdateIcon"))), CommandUpdate);

            SSLSSS Separator3 = new(Theme);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "ExitText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "ExitIcon"))), CommandClose);
        }

        public bool Dispose()
        {
            TrayIcon.Visible = false;
            TrayIcon.Dispose();

            return true;
        }

        public bool State()
        {
            return TrayIcon.Visible;
        }

        public bool Show()
        {
            return TrayIcon.Visible = true;
        }

        public bool Hide()
        {
            return TrayIcon.Visible = false;
        }

        private void ContextMenuAdjustment()
        {
            ContextMenu.Closed += (s, e) =>
            {
                ContextMenu.Hide();
                ContextMenu.Items.Clear();
                ContextMenu.Visible = false;
            };
        }

        private void MouseClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Right)
            {
                ContextMenu.Hide();

                Initialize();

                ContextMenu.Show(SSLHC.MenuPosition(ContextMenu));
            }
        }

        private void MouseDoubleClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Left)
            {
                SSLCI.Command();
            }
        }

        private void CommandInterface(object sender, EventArgs e)
        {
            SSLCI.Command();
        }

        private void CommandSetting(object sender, EventArgs e)
        {
            SSLCS.Command();
        }

        private void CommandUpdate(object sender, EventArgs e)
        {
            SSLCU.Command();
        }

        private void CommandEngine(object sender, EventArgs e)
        {
            SSLCE.Command();
        }

        private void CommandReport(object sender, EventArgs e)
        {
            SSLCR.Command();
        }

        private void CommandClose(object sender, EventArgs e)
        {
            SSLCC.Command();
        }
    }
}