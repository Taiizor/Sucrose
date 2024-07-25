using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Controls;

namespace Sucrose.Portal.Controls
{
    public class SizeConstrainingContainer : ContentControl
    {
        public SizeConstrainingContainer()
        {
            SizeChanged += SizeConstrainingContainer_SizeChanged;
            HorizontalAlignment = HorizontalAlignment.Left;
            VerticalAlignment = VerticalAlignment.Stretch;
        }

        private void SizeConstrainingContainer_SizeChanged(object sender, SizeChangedEventArgs e)
        {
            base.MeasureOverride(e.NewSize);
        }

        protected override Size MeasureOverride(Size constraint)
        {
            if (Content is UIElement content)
            {
                content.Measure(constraint);

                double width = Math.Min(content.DesiredSize.Width, constraint.Width);
                double height = Math.Min(content.DesiredSize.Height, constraint.Height);

                return new Size(width, height - height);
            }

            return base.MeasureOverride(constraint);
        }

        protected override Size ArrangeOverride(Size arrangeSize)
        {
            if (Content is UIElement content)
            {
                content.Arrange(new Rect(0, 0, arrangeSize.Width, arrangeSize.Height));
            }

            return base.ArrangeOverride(arrangeSize);
        }
    }

    public class ResizableContainer : DynamicScrollViewer
    {
        public ResizableContainer()
        {
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled;
            VerticalScrollBarVisibility = ScrollBarVisibility.Disabled;
            Content = new ContentPresenter();
        }

        protected override void OnContentChanged(object oldContent, object newContent)
        {
            base.OnContentChanged(oldContent, newContent);

            if (newContent is UIElement element)
            {
                element.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
                element.Arrange(new Rect(0, 0, element.DesiredSize.Width, element.DesiredSize.Height));

                this.MaxWidth = element.DesiredSize.Width;
                this.MaxHeight = element.DesiredSize.Height;
            }
        }
    }

    public class FittingContainer : ContentControl
    {
        public FittingContainer()
        {
            Loaded += FittingContainer_Loaded;
        }

        private void FittingContainer_Loaded(object sender, RoutedEventArgs e)
        {
            if (Content is UIElement content)
            {
                content.Measure(new Size(double.PositiveInfinity, double.PositiveInfinity));
                content.Arrange(new Rect(0, 0, content.DesiredSize.Width, content.DesiredSize.Height));

                this.MaxWidth = content.DesiredSize.Width;
                this.MaxHeight = content.DesiredSize.Height;
            }
        }
    }

    public class ContentFittingContainer : ContentControl
    {
        public ContentFittingContainer()
        {
            HorizontalAlignment = HorizontalAlignment.Stretch;
            VerticalAlignment = VerticalAlignment.Stretch;
        }

        protected override Size MeasureOverride(Size constraint)
        {
            if (Content is UIElement content)
            {
                content.Measure(constraint);

                double width = Math.Min(content.DesiredSize.Width, constraint.Width);
                double height = Math.Min(content.DesiredSize.Height, constraint.Height);

                return new Size(width, height);
            }

            return base.MeasureOverride(constraint);
        }

        protected override Size ArrangeOverride(Size arrangeSize)
        {
            if (Content is UIElement content)
            {
                content.Arrange(new Rect(0, 0, arrangeSize.Width, arrangeSize.Height));
            }

            return base.ArrangeOverride(arrangeSize);
        }
    }
}