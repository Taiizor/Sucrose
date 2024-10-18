﻿using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using Wpf.Ui.Controls;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// Pagination.xaml etkileşim mantığı
    /// </summary>
    public partial class Pagination : UserControl, IDisposable
    {
        public Pagination()
        {
            InitializeComponent();
            UpdatePagination();

            PageNumber.IconPlacement = ElementPlacement.Left;
            PageNumber.Icon = new SymbolIcon(SymbolRegular.DocumentPageBottomCenter24);
        }

        public event EventHandler SelectPageChanged;

        public bool EnabledJump
        {
            get;
            set
            {
                field = value;
                UpdatePagination();
            }
        } = true;

        public int MaxPage
        {
            get;
            set
            {
                if (value >= MinPage)
                {
                    field = value;
                }
                else
                {
                    field = MinPage;
                }
                UpdatePagination();
            }
        } = 1;

        public int MinPage
        {
            get;
            set
            {
                if (value <= MaxPage)
                {
                    field = value;
                }
                else
                {
                    field = MaxPage;
                }
                UpdatePagination();
            }
        } = 1;

        public int SelectPage
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    SelectPageChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = 1;

        private void Number_Click(object sender, RoutedEventArgs e)
        {
            ToggleButton Button = sender as ToggleButton;

            SelectPage = int.Parse(Button.Content.ToString());

            UpdateNumber();
        }

        private void PageBack_Click(object sender, RoutedEventArgs e)
        {
            if (SelectPage > MinPage)
            {
                SelectPage--;
                PageNumber.Value = SelectPage;
            }
        }

        private void PageNext_Click(object sender, RoutedEventArgs e)
        {
            if (SelectPage < MaxPage)
            {
                SelectPage++;
                PageNumber.Value = SelectPage;
            }
        }

        private void PageJump_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                SelectPage = (int)PageNumber.Value;
            }
            catch
            {
                PageNumber.Value = SelectPage;
            }
        }

        private void UpdateLineup()
        {
            if (MaxPage > 6)
            {
                if (SelectPage <= 4)
                {
                    Number1.Content = 1;
                    Number1.Visibility = Visibility.Visible;
                    Number2.Content = 2;
                    Number2.Visibility = Visibility.Visible;

                    Number3.Content = 3;
                    Number3.IsEnabled = true;
                    Number3.Visibility = Visibility.Visible;
                    Number4.Content = 4;
                    Number4.IsEnabled = true;
                    Number4.Visibility = Visibility.Visible;

                    Number5.Content = 5;
                    Number5.Visibility = Visibility.Visible;

                    Number6.Content = "...";
                    Number6.IsEnabled = false;
                    Number6.Visibility = Visibility.Visible;
                    Number7.Content = "...";
                    Number7.IsEnabled = false;
                    Number7.Visibility = Visibility.Visible;

                    Number8.Content = MaxPage - 1;
                    Number8.Visibility = Visibility.Visible;
                    Number9.Content = MaxPage;
                    Number9.Visibility = Visibility.Visible;
                }
                else if (MaxPage - SelectPage < 4)
                {
                    Number1.Content = 1;
                    Number1.Visibility = Visibility.Visible;
                    Number2.Content = 2;
                    Number2.Visibility = Visibility.Visible;

                    Number3.Content = "...";
                    Number3.IsEnabled = false;
                    Number3.Visibility = Visibility.Visible;
                    Number4.Content = "...";
                    Number4.IsEnabled = false;
                    Number4.Visibility = Visibility.Visible;

                    Number5.Content = MaxPage - 4;
                    Number5.Visibility = Visibility.Visible;

                    Number6.Content = MaxPage - 3;
                    Number6.IsEnabled = true;
                    Number6.Visibility = Visibility.Visible;
                    Number7.Content = MaxPage - 2;
                    Number7.IsEnabled = true;
                    Number7.Visibility = Visibility.Visible;

                    Number8.Content = MaxPage - 1;
                    Number8.Visibility = Visibility.Visible;
                    Number9.Content = MaxPage;
                    Number9.Visibility = Visibility.Visible;
                }
                else
                {
                    Number1.Content = 1;
                    Number1.Visibility = Visibility.Visible;
                    Number2.Content = 2;
                    Number2.Visibility = Visibility.Visible;

                    Number3.Content = "...";
                    Number3.IsEnabled = false;
                    Number3.Visibility = Visibility.Visible;

                    Number4.Content = SelectPage - 1;
                    Number4.IsEnabled = true;
                    Number4.Visibility = Visibility.Visible;
                    Number5.Content = SelectPage;
                    Number5.Visibility = Visibility.Visible;
                    Number6.Content = SelectPage + 1;
                    Number6.IsEnabled = true;
                    Number6.Visibility = Visibility.Visible;

                    Number7.Content = "...";
                    Number7.IsEnabled = false;
                    Number7.Visibility = Visibility.Visible;

                    Number8.Content = MaxPage - 1;
                    Number8.Visibility = Visibility.Visible;
                    Number9.Content = MaxPage;
                    Number9.Visibility = Visibility.Visible;
                }
            }
            else
            {
                Number1.Content = 1;
                Number2.Content = 2;
                Number3.Content = 3;
                Number3.IsEnabled = true;
                Number4.Content = 4;
                Number4.IsEnabled = true;
                Number5.Content = 5;
                Number6.Content = 6;
                Number6.IsEnabled = true;
                Number7.Content = 7;
                Number7.IsEnabled = true;
                Number8.Content = 8;
                Number9.Content = 9;

                foreach (UIElement Element in PagePanel.Children)
                {
                    if (Element is ToggleButton)
                    {
                        ToggleButton Button = Element as ToggleButton;

                        if (Convert.ToInt32(Button.Content) > MaxPage)
                        {
                            Button.Visibility = Visibility.Collapsed;
                        }
                        else
                        {
                            Button.Visibility = Visibility.Visible;
                        }
                    }
                }
            }
        }

        private void UpdateNumber()
        {
            foreach (UIElement Element in PagePanel.Children)
            {
                if (Element is ToggleButton)
                {
                    ToggleButton Button = Element as ToggleButton;

                    if (Button.IsEnabled && Convert.ToInt32(Button.Content) == SelectPage)
                    {
                        Button.IsChecked = true;
                    }
                    else
                    {
                        Button.IsChecked = false;
                    }
                }
            }
        }

        private void UpdatePagination()
        {
            if (MaxPage > MinPage)
            {
                Visibility = Visibility.Visible;

                if (SelectPage >= MaxPage)
                {
                    PageNext.IsEnabled = false;
                }
                else
                {
                    PageNext.IsEnabled = true;
                }

                if (SelectPage <= MinPage)
                {
                    PageBack.IsEnabled = false;
                }
                else
                {
                    PageBack.IsEnabled = true;
                }

                if (EnabledJump)
                {
                    PageNumber.Minimum = MinPage;
                    PageNumber.Maximum = MaxPage;
                    PageNumber.Value = SelectPage;

                    PageJump.Visibility = Visibility.Visible;
                    PageNumber.Visibility = Visibility.Visible;
                }
                else
                {
                    PageJump.Visibility = Visibility.Collapsed;
                    PageNumber.Visibility = Visibility.Collapsed;
                }

                UpdateLineup();

                UpdateNumber();
            }
            else
            {
                Visibility = Visibility.Collapsed;
            }

            if (SelectPage > MaxPage)
            {
                SelectPage = MaxPage;
            }
            else if (SelectPage < MinPage)
            {
                SelectPage = MinPage;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}