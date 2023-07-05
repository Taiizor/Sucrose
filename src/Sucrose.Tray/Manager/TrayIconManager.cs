using System.Globalization;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGHTL = Sucrose.Globalization.Helper.TrayLocalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using STCC = Sucrose.Tray.Command.Close;
using STCI = Sucrose.Tray.Command.Interface;
using STCR = Sucrose.Tray.Command.Report;
using STRDR = Sucrose.Tray.Renderer.DarkRenderer;
using STRLR = Sucrose.Tray.Renderer.LightRenderer;
using STSSS = Sucrose.Tray.Separator.StripSeparator;
using SSHC = Sucrose.Space.Helper.Command;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Tray.Manager
{
    public class TrayIconManager
    {
        private ContextMenuStrip ContextMenu { get; set; } = new();

        private NotifyIcon TrayIcon { get; set; } = new();

        private SEWTT ThemeType { get; set; }

        public void Start(SEWTT ThemeType, string CultureName)
        {
            SGMR.CultureInfo = new CultureInfo(CultureName, true);
            this.ThemeType = ThemeType;

            TrayIcon.Text = SGHTL.GetValue("TrayText");
            TrayIcon.Icon = new Icon(SGHTL.GetValue("TrayIcon"));

            TrayIcon.MouseClick += MouseClick;
            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseDoubleClick += MouseDoubleClick;

            TrayIcon.Visible = true;
        }

        public void Initialize()
        {
            if (ThemeType == SEWTT.Dark)
            {
                ContextMenu.Renderer = new STRDR();
            }
            else
            {
                ContextMenu.Renderer = new STRLR();
            }

            ContextMenu.Items.Add(SGHTL.GetValue("OpenText"), Image.FromFile(SGHTL.GetValue("OpenIcon")), CommandInterface);

            STSSS Separator1 = new(ThemeType);
            ContextMenu.Items.Add(Separator1.Strip);

            if (SSHC.Work(SMR.MediaElementLive))
            {
                ContextMenu.Items.Add(SGHTL.GetValue("WallCloseText"), null, CommandWallpaper);
                ContextMenu.Items.Add(SGHTL.GetValue("WallStartText"), null, null); //WallStopText

                if (true) //Engine is run and working
                {
                    ContextMenu.Items.Add(SGHTL.GetValue("WallChangeText"), null, null);
                    ContextMenu.Items.Add(SGHTL.GetValue("WallCustomizeText"), null, null);
                }
            }
            else
            {
                ContextMenu.Items.Add(SGHTL.GetValue("WallOpenText"), null, null);
            }

            STSSS Separator2 = new(ThemeType);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(SGHTL.GetValue("SettingsText"), Image.FromFile(SGHTL.GetValue("SettingsIcon")), null);
            ContextMenu.Items.Add(SGHTL.GetValue("ReportText"), Image.FromFile(SGHTL.GetValue("ReportIcon")), CommandReport);

            STSSS Separator3 = new(ThemeType);
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

                Point MousePosition = Control.MousePosition;

                MousePosition.Offset(-(ContextMenu.Size.Width / 2), -(30 + ContextMenu.Size.Height));

                ContextMenu.Show(MousePosition);
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

        private void CommandWallpaper(object sender, EventArgs e)
        {
            SSHC.Kill(SMR.MediaElementLive);
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