using System.IO;
using System.Reflection;
using ECT = Sucrose.Space.Enum.CommandsType;
using HC = Sucrose.Space.Helper.Command;
using MI = Sucrose.Space.Manage.Internal;
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
            ContextMenu.Items.Add("Duvar Kağıdını Kapat", null, null);
            ContextMenu.Items.Add("Duvar Kağıdını Durdur", null, null); //Başlat
            ContextMenu.Items.Add("Duvar Kağıdını Değiştir", null, null);
            ContextMenu.Items.Add("Duvar Kağıdını Özelleştir", null, null);
            //ContextMenu.Items.Add("Ayarlar", null, null);

            ContextMenuTheme.StripSeparatorCustom separator = new();

            ContextMenu.Items.Add(separator.stripSeparator);
            ContextMenu.Items.Add("Hata Bildir", Image.FromFile("Assets/Error.png"), null);
            ContextMenu.Items.Add(separator.stripSeparator);

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
                MessageBox.Show("Tray icon çift tıklandı!");
            }
        }

        private void CommandInterface(object sender, EventArgs e)
        {
            if (WPF != null)
            {
                string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

                HC.Run(Path.Combine(Folder, MI.ConsoleApplication), $"{MI.StartCommand}{ECT.Interface}{MI.ValueSeparator}{Path.Combine(Folder, MI.WPFApplication)}");
            }
            else if (WinForms != null)
            {
                string Folder = Path.GetDirectoryName(Assembly.GetEntryAssembly().Location);

                HC.Run(Path.Combine(Folder, MI.ConsoleApplication), $"{MI.StartCommand}{ECT.Interface}{MI.ValueSeparator}{Path.Combine(Folder, MI.WinFormsApplication)}");
            }
            else
            {
                MessageBox.Show("Arayüz uygulaması başlatılamadı!");
            }
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