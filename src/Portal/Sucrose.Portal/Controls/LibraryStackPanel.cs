using System.Windows;
using System.Windows.Controls;
using SPVCLC = Sucrose.Portal.Views.Controls.LibraryCard;

namespace Sucrose.Portal.Controls
{
    public class LibraryStackPanel : StackPanel, IDisposable
    {
        public static readonly DependencyProperty ItemMarginProperty = DependencyProperty.Register("ItemMargin", typeof(Thickness), typeof(LibraryStackPanel), new FrameworkPropertyMetadata(new Thickness(0)));

        public Thickness ItemMargin
        {
            get => (Thickness)GetValue(ItemMarginProperty);
            set => SetValue(ItemMarginProperty, value);
        }

        public static readonly DependencyProperty MaxItemsPerRowProperty = DependencyProperty.Register("MaxItemsPerRow", typeof(int), typeof(LibraryStackPanel), new FrameworkPropertyMetadata(int.MaxValue, MaxItemsPerRowPropertyChanged));

        public int MaxItemsPerRow
        {
            get => (int)GetValue(MaxItemsPerRowProperty);
            set => SetValue(MaxItemsPerRowProperty, value);
        }

        private static void MaxItemsPerRowPropertyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            (d as LibraryStackPanel)?.InvalidateMeasure();
        }

        protected override Size MeasureOverride(Size availableSize)
        {
            double x = 0;
            double y = 0;
            double rowHeight = 0;
            int itemsInCurrentRow = 0;

            if (InternalChildren.Count >= 0)
            {
                InternalChildren
                    .OfType<UIElement>()
                    .Where(Element => Element.Visibility != Visibility.Visible)
                    .ToList()
                    .ForEach(InternalChildren.Remove);

                foreach (UIElement child in InternalChildren)
                {
                    child.Measure(new Size(double.PositiveInfinity, availableSize.Height));
                    double childWidth = Math.Min((child as FrameworkElement)?.MaxWidth ?? double.PositiveInfinity, child.DesiredSize.Width) + ItemMargin.Left + ItemMargin.Right;

                    if (x + childWidth > availableSize.Width || (MaxItemsPerRow > 0 && itemsInCurrentRow >= MaxItemsPerRow))
                    {
                        x = 0;
                        y += rowHeight;

                        rowHeight = 0;

                        itemsInCurrentRow = 0;
                    }

                    rowHeight = Math.Max(rowHeight, child.DesiredSize.Height + ItemMargin.Top + ItemMargin.Bottom);

                    x += childWidth;

                    itemsInCurrentRow++;
                }
            }

            y += rowHeight;

            return new Size(availableSize.Width, y);
        }

        protected override Size ArrangeOverride(Size finalSize)
        {
            double yOffset = 0;
            double rowWidth = 0;
            double rowHeight = 0;
            int itemsInCurrentRow = 0;

            if (InternalChildren.Count >= 0)
            {
                for (int i = 0; i < InternalChildren.Count; i++)
                {
                    UIElement child = InternalChildren[i];

                    double childMinWidth = (child as FrameworkElement)?.MinWidth ?? 0;
                    double childMaxWidth = (child as FrameworkElement)?.MaxWidth ?? double.PositiveInfinity;
                    double childWidth = Math.Max(childMinWidth, Math.Min(childMaxWidth, child.DesiredSize.Width)) + ItemMargin.Left + ItemMargin.Right;
                    double childHeight = child.DesiredSize.Height + ItemMargin.Top + ItemMargin.Bottom;

                    if (rowWidth + childWidth > finalSize.Width || (MaxItemsPerRow > 0 && itemsInCurrentRow >= MaxItemsPerRow))
                    {
                        DistributeExtraSpace(finalSize.Width, rowWidth, itemsInCurrentRow, i - itemsInCurrentRow, i, yOffset, rowHeight);

                        yOffset += rowHeight;
                        rowWidth = 0;
                        rowHeight = 0;
                        itemsInCurrentRow = 0;
                    }

                    rowWidth += childWidth;
                    rowHeight = Math.Max(rowHeight, childHeight);
                    itemsInCurrentRow++;
                }

                if (itemsInCurrentRow > 0)
                {
                    DistributeExtraSpace(finalSize.Width, rowWidth, itemsInCurrentRow, InternalChildren.Count - itemsInCurrentRow, InternalChildren.Count, yOffset, rowHeight);
                }
            }

            return new Size(finalSize.Width, yOffset + rowHeight);
        }

        private void DistributeExtraSpace(double totalWidth, double rowWidth, int itemsInRow, int startIndex, int endIndex, double yOffset, double rowHeight)
        {
            double extraSpace = totalWidth - rowWidth;
            double extraSpacePerItem = extraSpace / itemsInRow;
            double xOffset = (extraSpace > 0) ? 0 : (totalWidth - rowWidth) / 2;

            for (int i = startIndex; i < endIndex; i++)
            {
                UIElement child = InternalChildren[i];

                if (child.Visibility != Visibility.Visible)
                {
                    continue;
                }

                double childMinWidth = (child as FrameworkElement)?.MinWidth ?? 0;
                double desiredWidth = child.DesiredSize.Width + ItemMargin.Left + ItemMargin.Right;
                double childMaxWidth = (child as FrameworkElement)?.MaxWidth ?? double.PositiveInfinity;
                double finalChildWidth = Math.Max(childMinWidth, Math.Min(childMaxWidth, desiredWidth + extraSpacePerItem));

                double childHeight = child.DesiredSize.Height;
                child.Arrange(new Rect(new Point(xOffset + ItemMargin.Left, yOffset + ItemMargin.Top), new Size(finalChildWidth - ItemMargin.Left - ItemMargin.Right, childHeight)));
                xOffset += finalChildWidth;
            }
        }

        public void Dispose()
        {
            if (InternalChildren.Count >= 0)
            {
                InternalChildren
                    .OfType<SPVCLC>()
                    .ToList()
                    .ForEach(Card => Card.Dispose());

                InternalChildren.Clear();
            }

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}