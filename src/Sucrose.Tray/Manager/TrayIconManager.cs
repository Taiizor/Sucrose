using System.Globalization;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGHTL = Sucrose.Globalization.Helper.TrayLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;
using STCC = Sucrose.Tray.Command.Close;
using STCI = Sucrose.Tray.Command.Interface;
using STCR = Sucrose.Tray.Command.Report;
using STHC = Sucrose.Tray.Helper.Calculate;
using STRDR = Sucrose.Tray.Renderer.DarkRenderer;
using STRLR = Sucrose.Tray.Renderer.LightRenderer;
using STSSS = Sucrose.Tray.Separator.StripSeparator;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Tray.Manager
{
    public class TrayIconManager
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private ContextMenuStrip ContextMenu { get; set; } = new();

        private NotifyIcon TrayIcon { get; set; } = new();

        public void Start()
        {
            TrayIcon.Text = SGHTL.GetValue("TrayText");
            TrayIcon.Icon = new Icon(SGHTL.GetValue("TrayIcon"));

            TrayIcon.MouseClick += MouseClick;
            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseDoubleClick += MouseDoubleClick;

            TrayIcon.Visible = true;
        }

        public void Initialize()
        {
            SGMR.CultureInfo = new CultureInfo(Culture, true);

            if (Theme == SEWTT.Dark)
            {
                ContextMenu.Renderer = new STRDR();
            }
            else
            {
                ContextMenu.Renderer = new STRLR();
            }

            ContextMenu.Items.Add(SGHTL.GetValue("OpenText"), Image.FromFile(SGHTL.GetValue("OpenIcon")), CommandInterface);

            STSSS Separator1 = new(Theme);
            ContextMenu.Items.Add(Separator1.Strip);

            if (SSHC.Work(SMMI.EngineSettingManager.GetSetting(SMC.App, SMR.EngineLive)))
            {
                ContextMenu.Items.Add(SGHTL.GetValue("WallCloseText"), null, CommandEngine);
                //ContextMenu.Items.Add(SGHTL.GetValue("WallStartText"), null, null); //WallStopText

                //ContextMenu.Items.Add(SGHTL.GetValue("WallChangeText"), null, null);
                ContextMenu.Items.Add(SGHTL.GetValue("WallCustomizeText"), null, null);
            }
            else
            {
                ContextMenu.Items.Add(SGHTL.GetValue("WallOpenText"), null, CommandEngine);
            }

            STSSS Separator2 = new(Theme);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(SGHTL.GetValue("SettingsText"), Image.FromFile(SGHTL.GetValue("SettingsIcon")), null);
            ContextMenu.Items.Add(SGHTL.GetValue("ReportText"), Image.FromFile(SGHTL.GetValue("ReportIcon")), CommandReport);

            STSSS Separator3 = new(Theme);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(SGHTL.GetValue("ExitText"), Image.FromFile(SGHTL.GetValue("ExitIcon")), CommandClose);
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

                ContextMenu.Show(STHC.MenuPosition(ContextMenu));
            }
        }

        private void MouseDoubleClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Left)
            {
                STCI.Command();
            }
        }

        private void CommandInterface(object sender, EventArgs e)
        {
            STCI.Command();
        }

        private void CommandEngine(object sender, EventArgs e)
        {
            string Live = SMMI.EngineSettingManager.GetSetting(SMC.App, SMR.EngineLive);

            if (SSHC.Work(Live))
            {
                SSHC.Kill(Live);
            }
            else
            {
                SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.TextEngineLive[Live]}");
            }
        }

        private void CommandReport(object sender, EventArgs e)
        {
            STCR.Command();
        }

        private void CommandClose(object sender, EventArgs e)
        {
            STCC.Command();
        }
    }
}