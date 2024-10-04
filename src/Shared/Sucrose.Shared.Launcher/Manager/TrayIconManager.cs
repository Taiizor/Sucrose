using System.IO;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHV = Skylark.Helper.Versionly;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSLCC = Sucrose.Shared.Launcher.Command.Close;
using SSLCE = Sucrose.Shared.Launcher.Command.Engine;
using SSLCF = Sucrose.Shared.Launcher.Command.Feedback;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLCRG = Sucrose.Shared.Launcher.Command.Reportdog;
using SSLCRT = Sucrose.Shared.Launcher.Command.Report;
using SSLCS = Sucrose.Shared.Launcher.Command.Setting;
using SSLCU = Sucrose.Shared.Launcher.Command.Update;
using SSLHC = Sucrose.Shared.Launcher.Helper.Calculate;
using SSLHR = Sucrose.Shared.Launcher.Helper.Radius;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using SSLPE = Sucrose.Shared.Launcher.Command.Property;
using SSLRDR = Sucrose.Shared.Launcher.Renderer.DarkRenderer;
using SSLRLR = Sucrose.Shared.Launcher.Renderer.LightRenderer;
using SSLSSS = Sucrose.Shared.Launcher.Separator.StripSeparator;
using SSSHA = Sucrose.Shared.Space.Helper.Assets;
using SSSHC = Sucrose.Shared.Space.Helper.Cycyling;
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
            TrayIcon.Icon = new Icon(SSSHA.Get(SRER.GetValue("Launcher", "TrayIcon")));
            TrayIcon.Text = SRER.GetValue("Launcher", "TrayText");

            TrayIcon.MouseDoubleClick += MouseDoubleClick;
            TrayIcon.ContextMenuStrip = ContextMenu;
            TrayIcon.MouseClick += MouseClick;

            TrayIcon.Visible = SMMM.Visible;

            ContextMenuAdjustment();

            SSLCE.Command(false);

            SSLCRG.Command();

            if (SMMM.AutoUpdate)
            {
                SSLCU.Command(false);
            }
        }

        public void Initialize()
        {
            Dispose();

            SRHR.SetLanguage(SMMM.Culture);

            SSLHR.Corner(ContextMenu);

            if (SSDMM.ThemeType == SEWTT.Dark)
            {
                ContextMenu.Renderer = new SSLRDR();
            }
            else
            {
                ContextMenu.Renderer = new SSLRLR();
            }

            ContextMenu.Items.Add(new ToolStripMenuItem(SRER.GetValue("Launcher", "OpenText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "OpenIcon"))), CommandInterface)
            {
                Enabled = !SSSHP.Work(SSSMI.Portal)
            });

            SSLSSS Separator1 = new(SSDMM.ThemeType);

            if (SSSHL.Run())
            {
                ToolStripItem Change = new ToolStripMenuItem();
                ToolStripItem Customize = new ToolStripMenuItem();
                ToolStripItem Wallpaper = new ToolStripMenuItem();

                if (SMMM.PausePerformance && SSSHP.Work(SSSMI.Backgroundog))
                {
                    Wallpaper = new ToolStripMenuItem($"{SRER.GetValue("Launcher", "WallCloseText")} ({SRER.GetValue("Launcher", "PausedText")})", Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallCloseIcon"))))
                    {
                        Enabled = false
                    };
                }
                else
                {
                    Wallpaper = new ToolStripMenuItem(SRER.GetValue("Launcher", "WallCloseText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallCloseIcon"))), CommandEngine);
                }

                ContextMenu.Items.Add(Separator1.Strip);

                ContextMenu.Items.Add(Wallpaper);

                if (Directory.Exists(SMMM.LibraryLocation))
                {
                    List<string> Themes = Directory.GetDirectories(SMMM.LibraryLocation).Select(Path.GetFileName).ToList();

                    if (Themes.Any())
                    {
                        Themes = Themes.Except(SMMM.DisableCycyling).ToList();

                        if (SMMM.Cycyling && (Themes.Count > 1 || (Themes.Count == 1 && !Themes.Contains(SMMM.LibrarySelected))))
                        {
                            if (SMMM.PausePerformance && SSSHP.Work(SSSMI.Backgroundog))
                            {
                                Change = new ToolStripMenuItem($"{SRER.GetValue("Launcher", "WallChangeText")} ({SRER.GetValue("Launcher", "PausedText")})", Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallChangeIcon"))))
                                {
                                    Enabled = false
                                };
                            }
                            else
                            {
                                Change = new ToolStripMenuItem(SRER.GetValue("Launcher", "WallChangeText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallChangeIcon"))), CommandChange);
                            }

                            ContextMenu.Items.Add(Change);
                        }
                    }
                }

                string PropertiesPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseProperties);

                if (File.Exists(PropertiesPath))
                {
                    string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                    if (File.Exists(InfoPath) && SSTHI.CheckJson(SSTHI.ReadInfo(InfoPath)))
                    {
                        SSTHI Info = SSTHI.ReadJson(InfoPath);

                        if (Info.Type == SSDEWT.Web)
                        {
                            if (SMMM.PausePerformance && SSSHP.Work(SSSMI.Backgroundog))
                            {
                                Customize = new ToolStripMenuItem($"{SRER.GetValue("Launcher", "WallCustomizeText")} ({SRER.GetValue("Launcher", "PausedText")})", Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallCustomizeIcon"))))
                                {
                                    Enabled = false
                                };
                            }
                            else
                            {
                                Customize = new ToolStripMenuItem(SRER.GetValue("Launcher", "WallCustomizeText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallCustomizeIcon"))), CommandProperty)
                                {
                                    Enabled = !SSSHP.Work(SSSMI.Property)
                                };
                            }

                            ContextMenu.Items.Add(Customize);
                        }
                    }
                }
            }
            else if (SMMI.LibrarySettingManager.CheckFile())
            {
                string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                if (File.Exists(InfoPath) && SSTHI.CheckJson(SSTHI.ReadInfo(InfoPath)))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        ToolStripItem Wallpaper = new ToolStripMenuItem();

                        if (SMMM.ClosePerformance && SSSHP.Work(SSSMI.Backgroundog))
                        {
                            Wallpaper = new ToolStripMenuItem($"{SRER.GetValue("Launcher", "WallOpenText")} ({SRER.GetValue("Launcher", "ClosedText")})", Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallOpenIcon"))))
                            {
                                Enabled = false
                            };
                        }
                        else
                        {
                            Wallpaper = new ToolStripMenuItem(SRER.GetValue("Launcher", "WallOpenText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "WallOpenIcon"))), CommandEngine);
                        }

                        ContextMenu.Items.Add(Separator1.Strip);

                        ContextMenu.Items.Add(Wallpaper);
                    }
                }
            }

            SSLSSS Separator2 = new(SSDMM.ThemeType);
            ContextMenu.Items.Add(Separator2.Strip);

            ContextMenu.Items.Add(new ToolStripMenuItem(SRER.GetValue("Launcher", "SettingText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "SettingIcon"))), CommandSetting)
            {
                Enabled = !SSSHP.Work(SSSMI.Portal)
            });

            ContextMenu.Items.Add(new ToolStripMenuItem(SRER.GetValue("Launcher", "ReportText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "ReportIcon"))), CommandReport)
            {
                Enabled = SSLMI.ReportBox
            });

            ContextMenu.Items.Add(new ToolStripMenuItem(SRER.GetValue("Launcher", "FeedbackText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "FeedbackIcon"))), CommandFeedback)
            {
                Enabled = SSLMI.FeedbackBox
            });

            ToolStripItem Update = new ToolStripMenuItem();

            if (SSSHP.Work(SSSMI.Update))
            {
                if (SMMM.UpdateState)
                {
                    Update = new ToolStripMenuItem(SRER.GetValue("Launcher", "UpdateText", "Done"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "UpdateIcon"))))
                    {
                        Enabled = false
                    };
                }
                else
                {
                    Update = new ToolStripMenuItem(SRER.GetValue("Launcher", "UpdateText", "Check"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "UpdateIcon"))))
                    {
                        Enabled = false
                    };
                }
            }
            else
            {
                Update = new ToolStripMenuItem(SRER.GetValue("Launcher", "UpdateText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "UpdateIcon"))), CommandUpdate);
            }

            ContextMenu.Items.Add(Update);

            SSLSSS Separator3 = new(SSDMM.ThemeType);
            ContextMenu.Items.Add(Separator3.Strip);

            ContextMenu.Items.Add(SRER.GetValue("Launcher", "ExitText"), Image.FromFile(SSSHA.Get(SRER.GetValue("Launcher", "ExitIcon"))), CommandClose);
        }

        public bool Release()
        {
            TrayIcon.Visible = false;
            TrayIcon.Dispose();
            Dispose();

            return true;
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

        private void CommandProperty(object sender, EventArgs e)
        {
            SSLPE.Command();
        }

        private void CommandFeedback(object sender, EventArgs e)
        {
            SSLCF.Command();
        }

        private void CommandSetting(object sender, EventArgs e)
        {
            SSLCS.Command();
        }

        private void CommandReport(object sender, EventArgs e)
        {
            SSLCRT.Command();
        }

        private void CommandUpdate(object sender, EventArgs e)
        {
            SSLCU.Command();
        }

        private void CommandEngine(object sender, EventArgs e)
        {
            SSLCE.Command();
        }

        private void CommandChange(object sender, EventArgs e)
        {
            SSSHC.Change();
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