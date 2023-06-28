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

namespace Sucrose.Tray.Manager
{
    public class TrayIconManager
    {
        private NotifyIcon TrayIcon { get; set; } = new();

        private ContextMenuStrip ContextMenu { get; set; } = new();

        public void Start(SEWTT ThemeType, string CultureName)
        {
            SGMR.CultureInfo = new CultureInfo(CultureName, true);

            TrayIcon.Text = SGHTL.GetValue("TrayText");
            TrayIcon.Icon = new Icon(SGHTL.GetValue("TrayIcon"));

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

            ContextMenu.Items.Add(SGHTL.GetValue("WallCloseText"), null, null); //SGHTL.GetValue("WallOpenText")
            ContextMenu.Items.Add(SGHTL.GetValue("WallStopText"), null, null); //SGHTL.GetValue("WallStartText")
            ContextMenu.Items.Add(SGHTL.GetValue("WallChangeText"), null, null);
            ContextMenu.Items.Add(SGHTL.GetValue("WallCustomizeText"), null, null);

            STSSS Separator2 = new(ThemeType);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(SGHTL.GetValue("SettingsText"), Image.FromFile(SGHTL.GetValue("SettingsIcon")), null);
            ContextMenu.Items.Add(SGHTL.GetValue("ReportText"), Image.FromFile(SGHTL.GetValue("ReportIcon")), CommandReport);

            STSSS Separator3 = new(ThemeType);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(SGHTL.GetValue("ExitText"), Image.FromFile(SGHTL.GetValue("ExitIcon")), CommandClose);

            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseClick += MouseClick;
            TrayIcon.MouseDoubleClick += MouseDoubleClick;

            TrayIcon.Visible = true;
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