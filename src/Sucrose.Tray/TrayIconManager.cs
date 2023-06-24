using System.IO;
using System.Reflection;
using ECT = Sucrose.Space.Enum.CommandsType;
using HC = Sucrose.Space.Helper.Command;
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
            Start(null, null);
        }

        public void StartWPF(WPF WPF = null)
        {
            Start(WPF, null);
        }

        public void StartWinForms(WinForms WinForms = null)
        {
            Start(null, WinForms);
        }

        private void Start(WPF WPF = null, WinForms WinForms = null)
        {
            this.WPF = WPF;
            this.WinForms = WinForms;

            TrayIcon.Text = "Sucrose Wallpaper";
            TrayIcon.Icon = new Icon("Assets/TrayIcon.ico");

            ContextMenu.Renderer = new ContextMenuTheme.RendererDark();

            ContextMenu.Items.Add("Sucrose'yi Aç", Image.FromFile("Assets/WideScreen.png"), CommandInterface);

            ContextMenuTheme.StripSeparatorCustom Separator1 = new();
            ContextMenu.Items.Add(Separator1.stripSeparator);

            ContextMenu.Items.Add("Duvar Kağıdını Kapat", null, null);
            ContextMenu.Items.Add("Duvar Kağıdını Durdur", null, null); //Başlat
            ContextMenu.Items.Add("Duvar Kağıdını Değiştir", null, null);
            ContextMenu.Items.Add("Duvar Kağıdını Özelleştir", null, null);

            ContextMenuTheme.StripSeparatorCustom Separator2 = new();
            ContextMenu.Items.Add(Separator2.stripSeparator);

            ContextMenu.Items.Add("Ayarlar", Image.FromFile("Assets/Setting.png"), null);
            ContextMenu.Items.Add("Hata Bildir", Image.FromFile("Assets/Error.png"), CommandReport);

            ContextMenuTheme.StripSeparatorCustom Separator3 = new();
            ContextMenu.Items.Add(Separator3.stripSeparator);

            ContextMenu.Items.Add("Çıkış", Image.FromFile("Assets/Close.png"), CommandClose);

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