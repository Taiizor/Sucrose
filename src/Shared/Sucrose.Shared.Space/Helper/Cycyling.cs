using System.IO;
using SECNT = Skylark.Enum.ClearNumericType;
using SETT = Skylark.Enum.TimeType;
using SHN = Skylark.Helper.Numeric;
using SHV = Skylark.Helper.Versionly;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMC = Sucrose.Manager.Manage.Cycling;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMCL = Sucrose.Memory.Manage.Constant.Library;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDETCT = Sucrose.Shared.Dependency.Enum.TransitionCycleType;
using SSDMMC = Sucrose.Shared.Dependency.Manage.Manager.Cycling;
using SSETTE = Skylark.Standard.Extension.Time.TimeExtension;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Cycyling
    {
        public static bool Check(bool Time = true)
        {
            if (Directory.Exists(SMML.Location))
            {
                List<string> Themes = Directory.GetDirectories(SMML.Location).Select(Path.GetFileName).ToList();

                if (Themes.Any())
                {
                    Themes = Themes.Except(SMMC.DisableCycyling).ToList();

                    if (SMMC.Cycyling && (Themes.Count > 1 || (Themes.Count == 1 && !Themes.Contains(SMML.Selected))) && (SMMC.PassingCycyling >= Converter(SMMC.CycylingTime) || !Time))
                    {
                        foreach (string Theme in Themes)
                        {
                            string ThemePath = Path.Combine(SMML.Location, Theme);
                            string InfoPath = Path.Combine(ThemePath, SMMRC.SucroseInfo);

                            if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                            {
                                if (SSTHI.ReadCheck(InfoPath))
                                {
                                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                                    if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                                    {
                                        if (Theme != SMML.Selected)
                                        {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }

                        return false;
                    }
                    else
                    {
                        return false;
                    }
                }
                else
                {
                    return false;
                }
            }
            else
            {
                return false;
            }
        }

        public static void Change()
        {
            if ((!SMMB.ClosePerformance && !SMMB.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                if (Directory.Exists(SMML.Location))
                {
                    List<string> Themes = Directory.GetDirectories(SMML.Location).Select(Path.GetFileName).ToList();

                    if (Themes.Any())
                    {
                        string LibrarySelected = SMML.Selected;

                        Themes = Themes.Where(Theme => !SMMC.DisableCycyling.Contains(Theme) || Theme == LibrarySelected).ToList();

                        if (Themes.Count > 1)
                        {
                            string Selected = string.Empty;

                            int Index = Themes.IndexOf(LibrarySelected);

                            switch (SSDMMC.TransitionCycleType)
                            {
                                case SSDETCT.Random:
                                    while (string.IsNullOrEmpty(Selected))
                                    {
                                        while (Index == Themes.IndexOf(LibrarySelected))
                                        {
                                            Index = SMMRG.Randomise.Next(Themes.Count);
                                        }

                                        string Current = Themes[Index];

                                        string ThemePath = Path.Combine(SMML.Location, Current);
                                        string InfoPath = Path.Combine(ThemePath, SMMRC.SucroseInfo);

                                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                                        {
                                            if (SSTHI.ReadCheck(InfoPath))
                                            {
                                                SSTHI Info = SSTHI.ReadJson(InfoPath);

                                                if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                                                {
                                                    Selected = Current;
                                                }
                                            }
                                        }
                                    }
                                    break;
                                case SSDETCT.Sequential:
                                    if (Index < 0 || Index >= Themes.Count)
                                    {
                                        Index = 0;
                                    }
                                    else
                                    {
                                        Index += 1;

                                        if (Index >= Themes.Count)
                                        {
                                            Index = 0;
                                        }
                                    }

                                    foreach (string Theme in Themes.Skip(Index))
                                    {
                                        string ThemePath = Path.Combine(SMML.Location, Theme);
                                        string InfoPath = Path.Combine(ThemePath, SMMRC.SucroseInfo);

                                        if (Directory.Exists(ThemePath) && File.Exists(InfoPath))
                                        {
                                            if (SSTHI.ReadCheck(InfoPath))
                                            {
                                                SSTHI Info = SSTHI.ReadJson(InfoPath);

                                                if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                                                {
                                                    Selected = Theme;

                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    break;
                                default:
                                    break;
                            }

                            if (!string.IsNullOrEmpty(Selected))
                            {
                                SMMI.CyclingSettingManager.SetSetting(SMMCC.PassingCycyling, 0);

                                SMMI.LibrarySettingManager.SetSetting(SMMCL.Selected, Selected);

                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Cycyling}{SMMRG.ValueSeparator}{SMMRG.Unknown}");
                            }
                        }
                    }
                }
            }
        }

        private static int Converter(int Time)
        {
            return Convert.ToInt32(SHN.Numeral(SSETTE.Convert(Time, SETT.Minute, SETT.Second), false, false, 0, '0', SECNT.None));
        }
    }
}