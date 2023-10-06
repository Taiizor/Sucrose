using System.IO;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHV = Skylark.Helper.Versionly;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSLCC = Sucrose.Shared.Launcher.Command.Close;
using SSLCE = Sucrose.Shared.Launcher.Command.Engine;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLCR = Sucrose.Shared.Launcher.Command.Report;
using SSLCS = Sucrose.Shared.Launcher.Command.Setting;
using SSLCU = Sucrose.Shared.Launcher.Command.Update;
using SSLHC = Sucrose.Shared.Launcher.Helper.Calculate;
using SSLHR = Sucrose.Shared.Launcher.Helper.Radius;
using SSLMM = Sucrose.Shared.Launcher.Manage.Manager;
using SSLRDR = Sucrose.Shared.Launcher.Renderer.DarkRenderer;
using SSLRLR = Sucrose.Shared.Launcher.Renderer.LightRenderer;
using SSLSSS = Sucrose.Shared.Launcher.Separator.StripSeparator;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHA = Sucrose.Shared.Space.Helper.Assets;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Shared.Launcher.Manager
{
    public class TrayIconManager : IDisposable
    {
        private ContextMenuStrip ContextMenu { get; set; } = new();

        private NotifyIcon TrayIcon { get; set; } = new()
        {
            Visible = true
        };

        public void Start()
        {
            TrayIcon.Text = SSRER.GetValue("Launcher", "TrayText");
            TrayIcon.Icon = new Icon(SSSHA.Get(SSRER.GetValue("Launcher", "TrayIcon")));

            TrayIcon.MouseClick += MouseClick;
            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseDoubleClick += MouseDoubleClick;

            TrayIcon.Visible = SMMM.Visible;

            ContextMenuAdjustment();

            SSLCE.Command(false);
        }

        public void Initialize()
        {
            Dispose();

            SSRHR.SetLanguage(SMMM.Culture);

            SSLHR.Corner(ContextMenu);

            if (SSLMM.ThemeType == SEWTT.Dark)
            {
                ContextMenu.Renderer = new SSLRDR();
            }
            else
            {
                ContextMenu.Renderer = new SSLRLR();
            }

            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "OpenText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "OpenIcon"))), CommandInterface);

            SSLSSS Separator1 = new(SSLMM.ThemeType);

            if (SSSHL.Run() && (!SMMM.PausePerformance || !SSSHP.Work(SSSMI.Backgroundog)))
            {
                ContextMenu.Items.Add(Separator1.Strip);

                ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallCloseText"), null, CommandEngine);
                //ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallStartText"), null, null); //WallStopText

                //ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallChangeText"), null, null);

                string PropertiesPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseProperties);

                if (File.Exists(PropertiesPath))
                {
                    string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                    if (File.Exists(InfoPath))
                    {
                        SSTHI Info = SSTHI.ReadJson(InfoPath);

                        if (Info.Type == SSDEWT.Web)
                        {
                            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallCustomizeText"), null, null);
                        }
                    }
                }
            }
            else if (SMMI.LibrarySettingManager.CheckFile() && ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog)))
            {
                string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        ContextMenu.Items.Add(Separator1.Strip);

                        ContextMenu.Items.Add(SSRER.GetValue("Launcher", "WallOpenText"), null, CommandEngine);
                    }
                }
            }

            SSLSSS Separator2 = new(SSLMM.ThemeType);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "SettingText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "SettingIcon"))), CommandSetting);
            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "ReportText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "ReportIcon"))), CommandReport);

            ToolStripMenuItem Update = new(SSRER.GetValue("Launcher", "UpdateText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "UpdateIcon"))), CommandUpdate);

            if (SSSHP.Work(SSSMI.Update))
            {
                Update.Enabled = false;
            }

            ContextMenu.Items.Add(Update);

            SSLSSS Separator3 = new(SSLMM.ThemeType);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(SSRER.GetValue("Launcher", "ExitText"), Image.FromFile(SSSHA.Get(SSRER.GetValue("Launcher", "ExitIcon"))), CommandClose);
        }

        public bool Release()
        {
            TrayIcon.Visible = false;
            TrayIcon.Dispose();
            Dispose();

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

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}