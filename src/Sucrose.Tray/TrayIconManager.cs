using Skylark.Enum;
using System.IO;
using System.Reflection;
using ECT = Sucrose.Space.Enum.CommandsType;
using HC = Sucrose.Space.Helper.Command;
using HTL = Sucrose.Globalization.Helper.TrayLocalization;
using R = Sucrose.Memory.Readonly;
using WinForms = System.Windows.Forms.Application;
using WPF = System.Windows.Application;

namespace Sucrose.Tray
{
    public class TrayIconManager
    {
        private WPF WPF { get; set; } = null;

        private WinForms WinForms { get; set; } = null;

        private NotifyIcon TrayIcon { get; set; } = new();

        private ContextMenuStrip ContextMenu { get; set; } = new();

        public void Start()
        {
            Start(null, null, WindowsThemeType.Light);
        }

        public void Start(WindowsThemeType ThemeType = WindowsThemeType.Light)
        {
            Start(null, null, ThemeType);
        }

        public void StartWPF(WPF WPF = null)
        {
            Start(WPF, null, WindowsThemeType.Light);
        }

        public void StartWPF(WPF WPF = null, WindowsThemeType ThemeType = WindowsThemeType.Light)
        {
            Start(WPF, null, ThemeType);
        }

        public void StartWinForms(WinForms WinForms = null)
        {
            Start(null, WinForms, WindowsThemeType.Light);
        }

        public void StartWinForms(WinForms WinForms = null, WindowsThemeType ThemeType = WindowsThemeType.Light)
        {
            Start(null, WinForms, ThemeType);
        }

        private void Start(WPF WPF = null, WinForms WinForms = null, WindowsThemeType ThemeType = WindowsThemeType.Light)
        {
            this.WPF = WPF;
            this.WinForms = WinForms;

            TrayIcon.Text = HTL.GetValue("TrayText");
            TrayIcon.Icon = new Icon(HTL.GetValue("TrayIcon"));

            if (ThemeType == WindowsThemeType.Dark)
            {
                ContextMenu.Renderer = new RendererDark();
            }
            else
            {
                ContextMenu.Renderer = new RendererLight();
            }

            ContextMenu.Items.Add(HTL.GetValue("OpenText"), Image.FromFile(HTL.GetValue("OpenIcon")), CommandInterface);

            StripSeparator Separator1 = new(ThemeType);
            ContextMenu.Items.Add(Separator1.Strip);

            ContextMenu.Items.Add(HTL.GetValue("WallCloseText"), null, null);
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

        private void MouseDoubleClick(object? sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Left)
            {
                CommandInterface(sender, e);
            }
        }

        private void CommandInterface(object sender, EventArgs e)
        {
            if (WPF != null)
            {
                string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

                HC.Run(Path.Combine(Folder, R.ConsoleApplication), $"{R.StartCommand}{ECT.Interface}{R.ValueSeparator}{Path.Combine(Folder, R.WPFApplication)}");
            }
            else if (WinForms != null)
            {
                string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

                HC.Run(Path.Combine(Folder, R.ConsoleApplication), $"{R.StartCommand}{ECT.Interface}{R.ValueSeparator}{Path.Combine(Folder, R.WinFormsApplication)}");
            }
            else
            {
                MessageBox.Show("Arayüz uygulaması başlatılamadı!");
            }
        }

        private void CommandReport(object sender, EventArgs e)
        {
            string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

            HC.Run(Path.Combine(Folder, R.ConsoleApplication), $"{R.StartCommand}{ECT.Report}{R.ValueSeparator}{R.ReportWebsite}");
        }

        private void CommandClose(object sender, EventArgs e)
        {
            if (WPF != null)
            {
                WPF.Current.MainWindow.Close();
                WPF.Current.Shutdown();
            }
            else if (WinForms != null)
            {
                WinForms.Exit();
            }
            else
            {
                Application.ExitThread();
                Environment.Exit(0);
                Application.Exit();
            }
        }
    }
}