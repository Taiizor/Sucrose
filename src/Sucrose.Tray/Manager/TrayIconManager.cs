using Skylark.Enum;
using Sucrose.Tray.Command;
using Sucrose.Tray.Renderer;
using Sucrose.Tray.Separator;
using System.Globalization;
using SGMR = Sucrose.Globalization.Manage.Resources;
using HTL = Sucrose.Globalization.Helper.TrayLocalization;

namespace Sucrose.Tray.Manager
{
    public class TrayIconManager
    {
        private NotifyIcon TrayIcon { get; set; } = new();

        private ContextMenuStrip ContextMenu { get; set; } = new();

        public void Start(WindowsThemeType ThemeType, string CultureName)
        {
            SGMR.CultureInfo = new CultureInfo(CultureName, true);

            TrayIcon.Text = HTL.GetValue("TrayText");
            TrayIcon.Icon = new Icon(HTL.GetValue("TrayIcon"));

            if (ThemeType == WindowsThemeType.Dark)
            {
                ContextMenu.Renderer = new DarkRenderer();
            }
            else
            {
                ContextMenu.Renderer = new LightRenderer();
            }

            ContextMenu.Items.Add(HTL.GetValue("OpenText"), Image.FromFile(HTL.GetValue("OpenIcon")), CommandInterface);

            StripSeparator Separator1 = new(ThemeType);
            ContextMenu.Items.Add(Separator1.Strip);

            ContextMenu.Items.Add(HTL.GetValue("WallCloseText"), null, null); //HTL.GetValue("WallOpenText")
            ContextMenu.Items.Add(HTL.GetValue("WallStopText"), null, null); //HTL.GetValue("WallStartText")
            ContextMenu.Items.Add(HTL.GetValue("WallChangeText"), null, null);
            ContextMenu.Items.Add(HTL.GetValue("WallCustomizeText"), null, null);

            StripSeparator Separator2 = new(ThemeType);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(HTL.GetValue("SettingsText"), Image.FromFile(HTL.GetValue("SettingsIcon")), null);
            ContextMenu.Items.Add(HTL.GetValue("ReportText"), Image.FromFile(HTL.GetValue("ReportIcon")), CommandReport);

            StripSeparator Separator3 = new(ThemeType);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(HTL.GetValue("ExitText"), Image.FromFile(HTL.GetValue("ExitIcon")), CommandClose);

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
                Interface.Command();
            }
        }

        private void CommandInterface(object sender, EventArgs e)
        {
            Interface.Command();
        }

        private void CommandReport(object sender, EventArgs e)
        {
            Report.Command();
        }

        private void CommandClose(object sender, EventArgs e)
        {
            Close.Command();
        }
    }
}