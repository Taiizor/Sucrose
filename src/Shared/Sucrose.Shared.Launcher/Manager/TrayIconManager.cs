using System.Globalization;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGHLL = Sucrose.Globalization.Helper.LauncherLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSLCC = Sucrose.Shared.Launcher.Command.Close;
using SSLCE = Sucrose.Shared.Launcher.Command.Engine;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLCR = Sucrose.Shared.Launcher.Command.Report;
using SSLHC = Sucrose.Shared.Launcher.Helper.Calculate;
using SSLRDR = Sucrose.Shared.Launcher.Renderer.DarkRenderer;
using SSLRLR = Sucrose.Shared.Launcher.Renderer.LightRenderer;
using SSLSSS = Sucrose.Shared.Launcher.Separator.StripSeparator;
using SSSHA = Sucrose.Shared.Space.Helper.Assets;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Launcher.Manager
{
    public class TrayIconManager
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static bool Visible => SMMI.LauncherSettingManager.GetSetting(SMC.Visible, true);

        private ContextMenuStrip ContextMenu { get; set; } = new();

        private NotifyIcon TrayIcon { get; set; } = new();

        public void Start()
        {
            TrayIcon.Text = SGHLL.GetValue("TrayText");
            TrayIcon.Icon = new Icon(SSSHA.Get(SGHLL.GetValue("TrayIcon")));

            TrayIcon.MouseClick += MouseClick;
            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseDoubleClick += MouseDoubleClick;

            TrayIcon.Visible = Visible;

            SSLCE.Command(false);
        }

        public void Initialize()
        {
            SGMR.CultureInfo = new CultureInfo(Culture, true);

            if (Theme == SEWTT.Dark)
            {
                ContextMenu.Renderer = new SSLRDR();
            }
            else
            {
                ContextMenu.Renderer = new SSLRLR();
            }

            ContextMenu.Items.Add(SGHLL.GetValue("OpenText"), Image.FromFile(SSSHA.Get(SGHLL.GetValue("OpenIcon"))), CommandInterface);

            SSLSSS Separator1 = new(Theme);

            if (SSSHL.Run())
            {
                ContextMenu.Items.Add(Separator1.Strip);

                ContextMenu.Items.Add(SGHLL.GetValue("WallCloseText"), null, CommandEngine);
                //ContextMenu.Items.Add(SGHLL.GetValue("WallStartText"), null, null); //WallStopText

                //ContextMenu.Items.Add(SGHLL.GetValue("WallChangeText"), null, null);
                ContextMenu.Items.Add(SGHLL.GetValue("WallCustomizeText"), null, null);
            }
            else if (SMMI.EngineSettingManager.CheckFile())
            {
                ContextMenu.Items.Add(Separator1.Strip);

                ContextMenu.Items.Add(SGHLL.GetValue("WallOpenText"), null, CommandEngine);
            }

            SSLSSS Separator2 = new(Theme);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(SGHLL.GetValue("SettingsText"), Image.FromFile(SSSHA.Get(SGHLL.GetValue("SettingsIcon"))), null);
            ContextMenu.Items.Add(SGHLL.GetValue("ReportText"), Image.FromFile(SSSHA.Get(SGHLL.GetValue("ReportIcon"))), CommandReport);

            SSLSSS Separator3 = new(Theme);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(SGHLL.GetValue("ExitText"), Image.FromFile(SSSHA.Get(SGHLL.GetValue("ExitIcon"))), CommandClose);
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

        private void MouseClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Right)
            {
                ContextMenu.Items.Clear();

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